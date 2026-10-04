//! A deliberately small HTTP/1.1 client for local AI services.
//!
//! Only plain `http://` to loopback addresses is supported: local providers
//! (inference snaps, llama.cpp, Ollama, LM Studio, Foundry Local...) serve on
//! the same machine, and refusing anything else guarantees prompts never leave
//! it. Each request uses its own connection (`Connection: close`), so there is
//! no pooling state to get wrong.

use std::{
    io::{self, BufRead, BufReader, Read, Write},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, Shutdown, SocketAddr, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

/// Largest response head (status line and headers) accepted.
const MAX_HEAD_BYTES: usize = 64 * 1024;
/// Largest buffered (non-streaming) body accepted.
pub(crate) const MAX_BODY_BYTES: u64 = 32 * 1024 * 1024;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
/// How often a blocked read wakes up to check for [`Abort::abort`].
///
/// Shutting a socket down from another thread wakes a blocked `recv` on Linux
/// and macOS but not on Windows, so reads never block longer than this.
const ABORT_POLL: Duration = Duration::from_millis(50);

/// A parsed `http://host[:port][/base]` URL pointing at this machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Endpoint {
    addrs: Vec<SocketAddr>,
    host_header: String,
    base_path: String,
}

impl Endpoint {
    /// Parses `url`, accepting only loopback hosts (`localhost`, `127.0.0.0/8`,
    /// `::1`). An unspecified bind address (`0.0.0.0`, `::`), as reported by a
    /// server listening on all interfaces, is reached through loopback.
    pub(crate) fn parse(url: &str) -> Result<Self, String> {
        let rest = url
            .trim()
            .strip_prefix("http://")
            .ok_or_else(|| format!("only http:// URLs are supported, got {url:?}"))?;
        let (authority, path) = match rest.find('/') {
            Some(i) => (&rest[..i], &rest[i..]),
            None => (rest, ""),
        };
        if authority.contains('@') {
            return Err("URLs with credentials are not supported".into());
        }
        let (host, port) = split_host_port(authority)?;
        let ips: Vec<IpAddr> = if host.eq_ignore_ascii_case("localhost") {
            vec![Ipv4Addr::LOCALHOST.into(), Ipv6Addr::LOCALHOST.into()]
        } else {
            let ip: IpAddr = host
                .parse()
                .map_err(|_| format!("host {host:?} is not localhost or an IP address"))?;
            match ip {
                IpAddr::V4(v4) if v4.is_unspecified() => vec![Ipv4Addr::LOCALHOST.into()],
                IpAddr::V6(v6) if v6.is_unspecified() => vec![Ipv6Addr::LOCALHOST.into()],
                ip if ip.is_loopback() => vec![ip],
                _ => {
                    return Err(format!(
                        "host {host:?} is not a loopback address; only local services are supported"
                    ))
                }
            }
        };
        let base_path = path
            .split(['?', '#'])
            .next()
            .unwrap_or_default()
            .trim_end_matches('/')
            .to_owned();
        Ok(Self {
            addrs: ips
                .into_iter()
                .map(|ip| SocketAddr::new(ip, port))
                .collect(),
            host_header: authority.to_owned(),
            base_path,
        })
    }

    /// Sends one request to `base_path + path` and reads the response head.
    ///
    /// `on_connected` receives an [`Abort`] handle as soon as the socket is
    /// open, before the request is written, so a caller can abort the exchange
    /// from another thread while it is blocked waiting for the response.
    /// `read_timeout` is the longest the server may stay silent.
    pub(crate) fn send(
        &self,
        method: &str,
        path: &str,
        body: Option<&[u8]>,
        read_timeout: Duration,
        on_connected: &mut dyn FnMut(&Abort),
    ) -> io::Result<Response> {
        let stream = self.connect()?;
        let abort = Abort {
            flag: Arc::new(AtomicBool::new(false)),
            socket: stream.try_clone().ok().map(Arc::new),
        };
        on_connected(&abort);
        stream.set_read_timeout(Some(ABORT_POLL.min(read_timeout)))?;
        stream.set_write_timeout(Some(read_timeout))?;
        stream.set_nodelay(true)?;
        let stream = Conn {
            stream,
            aborted: abort.flag,
            idle_timeout: read_timeout,
        };

        let mut head = format!(
            "{method} {}{path} HTTP/1.1\r\nHost: {}\r\nUser-Agent: rust_local_ai/{}\r\n\
             Accept: application/json, text/event-stream\r\nConnection: close\r\n",
            self.base_path,
            self.host_header,
            env!("CARGO_PKG_VERSION"),
        );
        if let Some(body) = body {
            head.push_str(&format!(
                "Content-Type: application/json\r\nContent-Length: {}\r\n",
                body.len()
            ));
        }
        head.push_str("\r\n");
        let mut stream = stream;
        stream.write_all(head.as_bytes())?;
        if let Some(body) = body {
            stream.write_all(body)?;
        }
        stream.flush()?;

        let mut reader = BufReader::new(stream);
        let (status, framing) = read_head(&mut reader)?;
        Ok(Response {
            status,
            body: Body {
                reader,
                framing,
                chunk_left: 0,
                done: false,
            },
        })
    }

    fn connect(&self) -> io::Result<TcpStream> {
        let mut last_error = None;
        for addr in &self.addrs {
            match TcpStream::connect_timeout(addr, CONNECT_TIMEOUT) {
                Ok(stream) => return Ok(stream),
                Err(error) => last_error = Some(error),
            }
        }
        Err(last_error.unwrap_or_else(|| io::Error::other("no address to connect to")))
    }
}

fn split_host_port(authority: &str) -> Result<(&str, u16), String> {
    let (host, port) = if let Some(rest) = authority.strip_prefix('[') {
        let end = rest
            .find(']')
            .ok_or_else(|| format!("invalid IPv6 host in {authority:?}"))?;
        (&rest[..end], rest[end + 1..].strip_prefix(':'))
    } else {
        match authority.rsplit_once(':') {
            Some((host, port)) => (host, Some(port)),
            None => (authority, None),
        }
    };
    if host.is_empty() {
        return Err("URL has no host".into());
    }
    let port = match port {
        Some(port) => port.parse().map_err(|_| format!("invalid port {port:?}"))?,
        None => 80,
    };
    Ok((host, port))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Framing {
    Length(u64),
    Chunked,
    UntilClose,
}

fn read_head(reader: &mut impl BufRead) -> io::Result<(u16, Framing)> {
    let mut consumed = 0;
    let mut line = String::new();
    let mut next_line = |line: &mut String| -> io::Result<()> {
        line.clear();
        let read = reader
            .take((MAX_HEAD_BYTES - consumed) as u64)
            .read_line(line)?;
        consumed += read;
        if read == 0 || !line.ends_with('\n') {
            return Err(invalid("truncated or oversized HTTP response head"));
        }
        Ok(())
    };

    next_line(&mut line)?;
    let status = line
        .strip_prefix("HTTP/1.")
        .and_then(|rest| rest.get(2..5))
        .and_then(|code| code.parse::<u16>().ok())
        .ok_or_else(|| invalid("malformed HTTP status line"))?;

    let mut framing = Framing::UntilClose;
    loop {
        next_line(&mut line)?;
        let header = line.trim_end();
        if header.is_empty() {
            break;
        }
        let Some((name, value)) = header.split_once(':') else {
            continue;
        };
        let value = value.trim();
        if name.eq_ignore_ascii_case("transfer-encoding") {
            if value
                .rsplit(',')
                .next()
                .is_some_and(|v| v.trim().eq_ignore_ascii_case("chunked"))
            {
                framing = Framing::Chunked;
            }
        } else if name.eq_ignore_ascii_case("content-length") && framing != Framing::Chunked {
            framing = Framing::Length(
                value
                    .parse()
                    .map_err(|_| invalid("invalid Content-Length"))?,
            );
        }
    }
    Ok((status, framing))
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

pub(crate) struct Response {
    pub(crate) status: u16,
    pub(crate) body: Body,
}

impl Response {
    /// Reads the whole body, up to [`MAX_BODY_BYTES`].
    pub(crate) fn read_body(mut self) -> io::Result<Vec<u8>> {
        let mut body = Vec::new();
        (&mut self.body)
            .take(MAX_BODY_BYTES + 1)
            .read_to_end(&mut body)?;
        if body.len() as u64 > MAX_BODY_BYTES {
            return Err(invalid("HTTP response body is too large"));
        }
        Ok(body)
    }
}

/// Aborts one request from another thread.
#[derive(Clone)]
pub(crate) struct Abort {
    flag: Arc<AtomicBool>,
    /// A clone of the socket, shut down to wake a blocked read immediately on
    /// platforms where that works; elsewhere the read notices within
    /// [`ABORT_POLL`].
    socket: Option<Arc<TcpStream>>,
}

impl Abort {
    /// Makes any read blocked on, or later issued for, this request fail.
    pub(crate) fn abort(&self) {
        self.flag.store(true, Ordering::Release);
        if let Some(socket) = &self.socket {
            let _ = socket.shutdown(Shutdown::Both);
        }
    }
}

/// A socket whose reads time out after `idle_timeout` of silence and fail
/// promptly once aborted, independent of the platform's socket semantics.
struct Conn {
    stream: TcpStream,
    aborted: Arc<AtomicBool>,
    idle_timeout: Duration,
}

impl Conn {
    fn check_aborted(&self) -> io::Result<()> {
        if self.aborted.load(Ordering::Acquire) {
            Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "request aborted",
            ))
        } else {
            Ok(())
        }
    }
}

impl Read for Conn {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let started = Instant::now();
        loop {
            self.check_aborted()?;
            match self.stream.read(buf) {
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                    ) =>
                {
                    if started.elapsed() >= self.idle_timeout {
                        return Err(io::Error::new(
                            io::ErrorKind::TimedOut,
                            "the server sent nothing before the timeout",
                        ));
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                result => {
                    // A shutdown from `Abort` surfaces as EOF or a reset.
                    self.check_aborted()?;
                    return result;
                }
            }
        }
    }
}

impl Write for Conn {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.check_aborted()?;
        self.stream.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

/// A response body with its framing (length, chunked or until close) decoded.
pub(crate) struct Body {
    reader: BufReader<Conn>,
    framing: Framing,
    chunk_left: u64,
    done: bool,
}

impl Body {
    fn read_chunk_size(&mut self) -> io::Result<u64> {
        let mut line = String::new();
        (&mut self.reader).take(1024).read_line(&mut line)?;
        if !line.ends_with('\n') {
            return Err(invalid("malformed chunk size"));
        }
        let size = line.trim().split(';').next().unwrap_or_default().trim();
        u64::from_str_radix(size, 16).map_err(|_| invalid("malformed chunk size"))
    }

    fn expect_crlf(&mut self) -> io::Result<()> {
        let mut crlf = [0; 2];
        self.reader.read_exact(&mut crlf)?;
        if crlf == *b"\r\n" {
            Ok(())
        } else {
            Err(invalid("missing CRLF after chunk"))
        }
    }
}

impl Read for Body {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.done || buf.is_empty() {
            return Ok(0);
        }
        match self.framing {
            Framing::UntilClose => {
                let read = self.reader.read(buf)?;
                self.done = read == 0;
                Ok(read)
            }
            Framing::Length(left) => {
                if left == 0 {
                    self.done = true;
                    return Ok(0);
                }
                let max = buf.len().min(usize::try_from(left).unwrap_or(usize::MAX));
                let read = self.reader.read(&mut buf[..max])?;
                if read == 0 {
                    return Err(io::ErrorKind::UnexpectedEof.into());
                }
                self.framing = Framing::Length(left - read as u64);
                Ok(read)
            }
            Framing::Chunked => {
                if self.chunk_left == 0 {
                    self.chunk_left = self.read_chunk_size()?;
                    if self.chunk_left == 0 {
                        // Skip trailers up to the terminating empty line.
                        let mut line = String::new();
                        loop {
                            line.clear();
                            let read = (&mut self.reader).take(8 * 1024).read_line(&mut line)?;
                            if read == 0 || line.trim_end().is_empty() {
                                break;
                            }
                        }
                        self.done = true;
                        return Ok(0);
                    }
                }
                let max = buf
                    .len()
                    .min(usize::try_from(self.chunk_left).unwrap_or(usize::MAX));
                let read = self.reader.read(&mut buf[..max])?;
                if read == 0 {
                    return Err(io::ErrorKind::UnexpectedEof.into());
                }
                self.chunk_left -= read as u64;
                if self.chunk_left == 0 {
                    self.expect_crlf()?;
                }
                Ok(read)
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::{
        io::{BufRead, BufReader, Read, Write},
        net::TcpListener,
        thread,
    };

    use super::*;

    /// Serves one connection with `response`, returning the raw request.
    pub(crate) fn serve_once(response: Vec<u8>) -> (String, thread::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/v1", listener.local_addr().unwrap());
        let handle = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut request = String::new();
            let mut length = 0;
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if let Some(v) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                    length = v.trim().parse().unwrap();
                }
                request.push_str(&line);
                if line == "\r\n" {
                    break;
                }
            }
            let mut body = vec![0; length];
            reader.read_exact(&mut body).unwrap();
            request.push_str(&String::from_utf8(body).unwrap());
            (&stream).write_all(&response).unwrap();
            request
        });
        (url, handle)
    }

    #[test]
    fn parses_only_loopback_urls() {
        let e = Endpoint::parse("http://localhost:8338/v1/").unwrap();
        assert_eq!(e.base_path, "/v1");
        assert_eq!(e.addrs.len(), 2);
        assert_eq!(e.addrs[0].port(), 8338);
        let e = Endpoint::parse("http://0.0.0.0:9090").unwrap();
        assert_eq!(e.addrs, vec!["127.0.0.1:9090".parse().unwrap()]);
        let e = Endpoint::parse("http://[::1]:1234/v3").unwrap();
        assert_eq!(e.addrs, vec!["[::1]:1234".parse().unwrap()]);
        assert_eq!(e.host_header, "[::1]:1234");
        assert_eq!(
            Endpoint::parse("http://127.0.0.1").unwrap().addrs[0].port(),
            80
        );

        for bad in [
            "https://localhost:1/v1",
            "http://example.com/v1",
            "http://192.168.1.10:8080",
            "http://user:pw@localhost:1",
            "http://localhost:notaport",
            "http://:80",
            "ftp://localhost",
        ] {
            assert!(Endpoint::parse(bad).is_err(), "{bad} should be rejected");
        }
    }

    fn get(response: &str) -> (Result<(u16, Vec<u8>), io::Error>, String) {
        let (url, server) = serve_once(response.as_bytes().to_vec());
        let result = Endpoint::parse(&url)
            .unwrap()
            .send("GET", "/models", None, Duration::from_secs(5), &mut |_| {})
            .and_then(|r| {
                let status = r.status;
                r.read_body().map(|b| (status, b))
            });
        (result, server.join().unwrap())
    }

    #[test]
    fn decodes_content_length_chunked_and_close_framing() {
        let (result, request) = get("HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhelloEXTRA");
        assert_eq!(result.unwrap(), (200, b"hello".to_vec()));
        assert!(request.starts_with("GET /v1/models HTTP/1.1\r\n"));
        assert!(request.contains("Connection: close\r\n"));

        let (result, _) = get("HTTP/1.1 201 Created\r\nTransfer-Encoding: chunked\r\n\r\n\
             4;ext=1\r\nWiki\r\n5\r\npedia\r\n0\r\nTrailer: x\r\n\r\n");
        assert_eq!(result.unwrap(), (201, b"Wikipedia".to_vec()));

        let (result, _) = get("HTTP/1.0 404 Not Found\r\n\r\nmissing");
        assert_eq!(result.unwrap(), (404, b"missing".to_vec()));
    }

    #[test]
    fn rejects_malformed_responses() {
        for response in [
            "garbage\r\n\r\n",
            "HTTP/1.1 200 OK\r\nContent-Length: 10\r\n\r\nshort",
            "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\nzz\r\n",
            "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nabcXY",
            "HTTP/1.1 200 OK\r\nContent-Length: nope\r\n\r\n",
        ] {
            assert!(get(response).0.is_err(), "{response:?} should fail");
        }
    }

    #[test]
    fn sends_json_bodies() {
        let (url, server) = serve_once(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}".to_vec());
        let response = Endpoint::parse(&url)
            .unwrap()
            .send(
                "POST",
                "/chat/completions",
                Some(b"{\"a\":1}"),
                Duration::from_secs(5),
                &mut |_| {},
            )
            .unwrap();
        assert_eq!(response.read_body().unwrap(), b"{}");
        let request = server.join().unwrap();
        assert!(request.starts_with("POST /v1/chat/completions HTTP/1.1\r\n"));
        assert!(request.contains("Content-Type: application/json\r\nContent-Length: 7\r\n"));
        assert!(request.ends_with("\r\n\r\n{\"a\":1}"));
    }

    #[test]
    fn connection_refused_is_an_error() {
        let port = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let endpoint = Endpoint::parse(&format!("http://127.0.0.1:{port}")).unwrap();
        assert!(endpoint
            .send("GET", "/", None, Duration::from_secs(1), &mut |_| {})
            .is_err());
    }

    /// A server that accepts the request and then never answers.
    fn silent_server() -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/v1", listener.local_addr().unwrap());
        let handle = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut byte = [0; 1];
            // Hold the connection open until the client goes away.
            while matches!((&stream).read(&mut byte), Ok(1)) {}
        });
        (url, handle)
    }

    /// Sends to a silent server, aborting through `make_handle(abort)` after
    /// 200 ms, and returns the error and how long the request blocked.
    fn abort_silent_request(
        make_handle: fn(&Abort) -> Abort,
        idle_timeout: Duration,
    ) -> (io::Error, Duration) {
        let (url, server) = silent_server();
        let started = Instant::now();
        let error = Endpoint::parse(&url)
            .unwrap()
            .send(
                "POST",
                "/chat/completions",
                Some(b"{}"),
                idle_timeout,
                &mut |abort| {
                    let handle = make_handle(abort);
                    thread::spawn(move || {
                        thread::sleep(Duration::from_millis(200));
                        handle.abort();
                    });
                },
            )
            .err()
            .expect("an aborted request must fail");
        let elapsed = started.elapsed();
        server.join().unwrap();
        (error, elapsed)
    }

    #[test]
    fn abort_unblocks_a_request_waiting_for_the_server() {
        let (error, elapsed) = abort_silent_request(Abort::clone, Duration::from_secs(600));
        assert_eq!(error.kind(), io::ErrorKind::ConnectionAborted, "{error}");
        assert!(elapsed < Duration::from_secs(5), "abort took {elapsed:?}");
    }

    #[test]
    fn abort_works_even_when_shutdown_does_not_wake_the_read() {
        // On Windows, shutting the socket down from another thread does not
        // wake a blocked read. Abort through the flag alone to prove the
        // polling path unblocks the request on every platform.
        fn flag_only(abort: &Abort) -> Abort {
            Abort {
                flag: abort.flag.clone(),
                socket: None,
            }
        }
        let (error, elapsed) = abort_silent_request(flag_only, Duration::from_secs(600));
        assert_eq!(error.kind(), io::ErrorKind::ConnectionAborted, "{error}");
        assert!(elapsed < Duration::from_secs(5), "abort took {elapsed:?}");
    }

    #[test]
    fn a_silent_server_still_times_out() {
        let (url, server) = silent_server();
        let started = Instant::now();
        let error = Endpoint::parse(&url)
            .unwrap()
            .send(
                "GET",
                "/models",
                None,
                Duration::from_millis(300),
                &mut |_| {},
            )
            .err()
            .expect("a silent server must time out");
        assert_eq!(error.kind(), io::ErrorKind::TimedOut, "{error}");
        assert!(started.elapsed() < Duration::from_secs(5));
        drop(error);
        server.join().unwrap();
    }
}
