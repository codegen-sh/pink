# Python bindings for Pink

## Requirements

- Maturin

## Usage

### With Maturin (recommended)

```bash
uv venv
source .venv/bin/activate
maturin develop
```

### Without Maturin

It's much slower to compile without maturin, but it can be used as a normal package.

```bash
uv venv
source .venv/bin/activate
uv sync
```
