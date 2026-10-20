#!/usr/bin/env bash
# Builds the Python module `golden` from golden/schema/golden.bitis with the bitis compiler of
# this repository (bitis compile -l python, which runs `maturin develop`) and installs it into
# the venv (see golden/README.md).
#
#   golden/python/build.sh
#
# Environment:
#   BITIS_PYTHON_VENV       venv with maturin and pytest (default: the active venv, $VIRTUAL_ENV)
#   BITIS_GOLDEN_BUILD_DIR  build directory (default: target/golden); the maturin project is
#                           written to $BITIS_GOLDEN_BUILD_DIR/python/golden
#   CARGO_FLAGS             extra cargo flags, e.g. "--offline"
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
BUILD=${BITIS_GOLDEN_BUILD_DIR:-$ROOT/target/golden}
VENV=${BITIS_PYTHON_VENV:-${VIRTUAL_ENV:-}}
if [[ -z "$VENV" || ! -x "$VENV/bin/python" ]]; then
    echo "no venv: set BITIS_PYTHON_VENV or activate a venv with maturin and pytest" >&2
    exit 2
fi
export VIRTUAL_ENV=$VENV
export PATH="$VENV/bin:$PATH"

# shellcheck disable=SC2086
(cd "$ROOT" && cargo build -q ${CARGO_FLAGS:-} --bin bitis)

PROJ=$BUILD/python/golden
mkdir -p "$PROJ/src" "$PROJ/golden"
# the maturin project (what `bitis setup -s maturin` + `maturin init` create, with a path
# dependency on this repository instead of the bitis release)
cat > "$PROJ/Cargo.toml.new" <<TOML
[package]
name = "golden"
version = "0.0.0"
edition = "2021"

[lib]
name = "golden"
crate-type = ["cdylib"]

[dependencies]
pyo3 = "0"
bitis = { path = "$ROOT" }

[workspace]
TOML
if [[ ! -f "$PROJ/Cargo.toml" ]] || ! grep -q "path = \"$ROOT\"" "$PROJ/Cargo.toml"; then
    mv "$PROJ/Cargo.toml.new" "$PROJ/Cargo.toml"
else
    rm "$PROJ/Cargo.toml.new"
fi
cat > "$PROJ/pyproject.toml" <<TOML
[build-system]
requires = ["maturin>=1.9,<2.0"]
build-backend = "maturin"

[project]
name = "golden"
requires-python = ">=3.8"
dynamic = ["version"]
TOML
echo "from .golden import *" > "$PROJ/golden/__init__.py"
: > "$PROJ/golden/py.typed"

# generates src/*.rs and golden/golden.pyi, updates Cargo.toml (version, pyo3, strum_macros),
# runs maturin develop
"$ROOT/target/debug/bitis" compile -l python -i "$ROOT/golden/schema/golden.bitis" -o "$PROJ" \
    --prevent-update-bitis-lib-in-crate
