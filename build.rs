use std::{env, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=native/apple.swift");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos") {
        return;
    }

    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    let library = out.join("librust_local_ai_apple.a");
    let module_cache = out.join("module-cache");
    let cargo_target = env::var("TARGET").expect("TARGET is set by Cargo");
    let swift_target = cargo_target.replace("-apple-darwin", "-apple-macosx11.0");
    let status = Command::new("swiftc")
        .env("CLANG_MODULE_CACHE_PATH", &module_cache)
        .args([
            "-parse-as-library",
            "-emit-library",
            "-static",
            "-O",
            "-module-name",
            "RustLocalAiApple",
            "-target",
            &swift_target,
            "native/apple.swift",
            "-o",
        ])
        .arg(&library)
        .status()
        .expect("failed to invoke swiftc for the Apple Foundation Models bridge");
    assert!(status.success(), "Swift Apple bridge failed to compile");

    println!("cargo:rustc-link-search=native={}", out.display());
    let swiftc = Command::new("xcrun")
        .args(["--find", "swiftc"])
        .output()
        .expect("failed to locate swiftc through xcrun");
    assert!(swiftc.status.success(), "xcrun could not locate swiftc");
    let swiftc = PathBuf::from(
        String::from_utf8(swiftc.stdout)
            .expect("xcrun returned a non-UTF-8 swiftc path")
            .trim(),
    );
    let swift_runtime = swiftc
        .parent()
        .and_then(|path| path.parent())
        .expect("swiftc is not inside a Swift toolchain usr/bin directory")
        .join("lib/swift/macosx");
    println!("cargo:rustc-link-search=native={}", swift_runtime.display());
    println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
    println!(
        "cargo:rustc-link-arg=-Wl,-rpath,{}",
        swift_runtime.display()
    );
    println!("cargo:rustc-link-lib=static=rust_local_ai_apple");
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=dylib=swiftCore");
    println!("cargo:rustc-link-lib=dylib=swift_Concurrency");
}
