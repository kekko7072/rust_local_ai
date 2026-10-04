# Contributing to rust_local_ai

Thank you for helping. This package is part of
[Universal Local AI](https://github.com/kekko7072/universal_local_ai), and its
[contributing guide](https://github.com/kekko7072/universal_local_ai/blob/main/CONTRIBUTING.md)
applies here too.

## Development

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
```

See the README for the support matrix and the release process.

## AI-assisted contributions

AI coding assistants (Claude, Copilot, Cursor and others) are welcome as
tools. The person who submits a change is its only author and is responsible
for it:

- **Commit as yourself.** The commit author and committer are the human
  contributor. Never commit as an AI identity such as
  `Claude <noreply@anthropic.com>`.
- **No AI attribution.** Commit messages, pull request titles and
  descriptions, changelogs and release notes must not credit an AI tool. That
  means no `Co-Authored-By:` trailers for AI assistants, no
  "Generated with ..." lines, and no assistant session links.
- **Review before you submit.** Read, test and understand every AI-written
  line as if you had written it yourself.
