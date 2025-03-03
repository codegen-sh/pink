# Pink

This is a rewrite of the core of the [Codegen SDK](https://github.com/codegen-sh/codegen) in Rust.

## Goals

- Support more languages
- Faster parse and execute time
- More memory efficient
- Incremental compilation

## Structure

- `codegen-sdk-common`: A crate that contains the common code for the SDK.
- `codegen-sdk-cst`: Definitions and utilities for the CST.
- `codegen-sdk-ast`: Definitions and utilities for the AST.
- `codegen-sdk-cst-generator`: A crate that generates the CST for the SDK.
- `codegen-sdk-ast-generator`: A crate that generates the AST and queries for the SDK. This requires the ts_query CST language to be generated first.
- `languages/*`: A crate for each language that contains the language-specific code for the SDK. It's largely boilerplate, most of the work is done by the `codegen-sdk-ast-generator` and `codegen-sdk-cst-generator` crates. These are split out to make compiling the SDK faster.
- `codegen-sdk-analyzer`: A crate that contains the core logic for the Incremenetal computation and state management of the SDK.
- `src`: A base program that uses the SDK.

## Development

### Installing Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install nightly
```

### Installing tools

```bash
cargo install cargo-binstall -y
cargo binstall cargo-nextest -y
cargo binstall cargo-insta -y
```

### Building the project

```bash
cargo build
```

### Running tests

```bash
cargo insta run --workspace --review
```

Some of the tests use snapshots managed by [Insta](https://insta.rs/docs/cli/).

### Running sample program

```bash
RUST_LOG=info cargo run --release /path/to/repo
```
