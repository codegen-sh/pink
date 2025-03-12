# Pink

[![CodSpeed Badge](https://img.shields.io/endpoint?url=https://codspeed.io/badge.json)](https://codspeed.io/codegen-sh/pink)

This is a rewrite of the core of the [Codegen SDK](https://github.com/codegen-sh/codegen) in Rust. The codegen-sdk is a multi-lingual type analysis and refactoring engine. It's designed to be used for providing tools and context to AI agents.

## Features

- Supports multiple languages
- Much faster parsing (~10x) of repositories
- Fast execution time using pre-parsed caches
- More memory efficient (50% memory usage compared to the python SDK)
- Incremental re-computation in response to changes made by the user or the tool

## Structure

- `codegen-sdk-common`: A crate that contains the common code for the SDK.
- `codegen-sdk-cst`: Definitions and utilities for the CST.
- `codegen-sdk-ast`: Definitions and utilities for the AST.
- `codegen-sdk-resolution`: Definitions and utilities for the AST.
- `codegen-sdk-cst-generator`: A crate that generates the CST for the SDK.
- `codegen-sdk-ast-generator`: A crate that generates the AST and queries for the SDK. This requires the ts_query CST language to be generated first.
- `codegen-sdk-bindings-generator`: A crate that generates python bindings to the rust library
- `languages/*`: A crate for each language that contains the language-specific code for the SDK. It's largely boilerplate, most of the work is done by the `codegen-sdk-ast-generator` and `codegen-sdk-cst-generator` crates. These are split out to make compiling the SDK faster.
- `codegen-sdk-analyzer`: A crate that contains the core logic for the Incremenetal computation and state management of the SDK.
- `bindings/python`: Python bindings for the library
- `src`: A base program that uses the SDK.

## Development

### Installing Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install nightly
```

### Installing tools

```bash
cargo install --no-confirm cargo-binstall
cargo binstall --no-confirm cargo-nextest
cargo binstall --no-confirm cargo-insta
```

### Building the project

```bash
cargo build --features stable
```

### Running tests

```bash
cargo insta run --workspace --review --features stable
```

Some of the tests use snapshots managed by [Insta](https://insta.rs/docs/cli/).

### Running sample program

```bash
RUST_LOG=info cargo run --release /path/to/repo
```
