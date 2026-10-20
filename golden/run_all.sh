#!/usr/bin/env bash
# Runs the golden vector tests of all runtimes (Rust, C++, Dart) against
# golden/vectors/manifest.txt (see golden/README.md).
#
#   golden/run_all.sh [--no-asan]
#
# Environment:
#   CARGO_FLAGS                       extra cargo flags, e.g. "--offline"
#   FETCHCONTENT_SOURCE_DIR_GOOGLETEST local googletest checkout for offline C++ builds
#   BITIS_GOLDEN_BUILD_DIR            C++ build directory (default: target/golden)
#   CXX                               C++ compiler (cmake default otherwise)
#   DART                              dart executable (default: dart on PATH, else
#                                     /home/dheide/sdk/flutter/bin/dart if it exists;
#                                     the Dart steps are skipped if none is found)
#   DART_PUB_FLAGS                    extra flags for dart pub get, e.g. "--offline"
#   CHROME_EXECUTABLE                 Chrome for the web runs (default: google-chrome,
#                                     chromium or chromium-browser on PATH; the web runs
#                                     are skipped if none is found)
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
BUILD=${BITIS_GOLDEN_BUILD_DIR:-$ROOT/target/golden}
CARGO_FLAGS=${CARGO_FLAGS:-}
ASAN=1
for a in "$@"; do
    case "$a" in
        --no-asan) ASAN=0 ;;
        *) echo "unknown option $a"; exit 2 ;;
    esac
done
# needed in some environments (preloaded libraries); harmless otherwise. Also needed while
# building because gtest_discover_tests runs the test binary.
export ASAN_OPTIONS=${ASAN_OPTIONS:-verify_asan_link_order=0}

declare -a RESULTS
run_step() {
    local name=$1; shift
    echo
    echo "=== $name"
    if "$@"; then RESULTS+=("PASS  $name")
    elif [[ $? == 77 ]]; then RESULTS+=("SKIP  $name")
    else RESULTS+=("FAIL  $name"); fi
}

# --- manifest up to date with the generator (Rust reference)
check_manifest() {
    # shellcheck disable=SC2086
    cargo run -q $CARGO_FLAGS --manifest-path "$ROOT/golden/gen_rust/Cargo.toml" -- --check
}

# --- Rust runtime
rust_tests() {
    # shellcheck disable=SC2086
    (cd "$ROOT" && cargo test -q $CARGO_FLAGS --test golden --test crc32)
}

# --- C++ runtime (gtest target bitis_golden in cpp_lib/tests)
cpp_tests() {
    local dir=$1; shift
    local args=(-S "$ROOT/cpp_lib" -B "$dir" "$@")
    if [[ -n "${FETCHCONTENT_SOURCE_DIR_GOOGLETEST:-}" ]]; then
        args+=("-DFETCHCONTENT_SOURCE_DIR_GOOGLETEST=$FETCHCONTENT_SOURCE_DIR_GOOGLETEST")
    fi
    cmake "${args[@]}" > "$dir.cmake.log" 2>&1 || { cat "$dir.cmake.log"; return 1; }
    cmake --build "$dir" --target bitis_golden -j "$(nproc 2>/dev/null || echo 4)" || return 1
    "$dir/tests/bitis_golden" --gtest_brief=1
}

# --- Python (bindings generated via maturin from the Rust code)
python_tests() {
    echo "SKIP: the Python bindings are built with maturin (not available here); they use the"
    echo "      Rust runtime, see golden/README.md"
    return 77
}

# --- Dart runtime (dart/bitis) and generated code (golden/gen/dart): package golden/dart,
# on the VM, in Chrome compiled with dart2js and in Chrome compiled with dart2wasm
DART_BIN=
DART_READY=0
find_dart() {
    if [[ -n "${DART:-}" ]]; then
        DART_BIN=$(command -v "$DART" 2>/dev/null) || { echo "DART=$DART is not executable"; return 1; }
    elif command -v dart > /dev/null 2>&1; then
        DART_BIN=$(command -v dart)
    elif [[ -x /home/dheide/sdk/flutter/bin/dart ]]; then
        DART_BIN=/home/dheide/sdk/flutter/bin/dart
    else
        echo "SKIP: dart not found (set DART or put dart on PATH)"
        return 77
    fi
}
dart_setup() {
    find_dart || return $?
    echo "using $DART_BIN: $("$DART_BIN" --version 2>&1)"
    # shellcheck disable=SC2086
    (cd "$ROOT/golden/dart" \
        && "$DART_BIN" pub get ${DART_PUB_FLAGS:-} \
        && "$DART_BIN" run tool/embed_manifest.dart --check \
        && "$DART_BIN" analyze --fatal-infos) || return 1
    DART_READY=1
}
have_chrome() {
    if [[ -n "${CHROME_EXECUTABLE:-}" ]]; then return 0; fi
    local c
    for c in google-chrome chromium chromium-browser; do
        if command -v "$c" > /dev/null 2>&1; then return 0; fi
    done
    return 1
}
dart_tests() {
    if [[ $DART_READY != 1 ]]; then echo "SKIP: dart not available or setup failed"; return 77; fi
    if [[ $1 == chrome ]] && ! have_chrome; then
        echo "SKIP: no Chrome found (set CHROME_EXECUTABLE)"
        return 77
    fi
    local args=(-p "$1")
    if [[ -n "${2:-}" ]]; then args+=(--compiler "$2"); fi
    (cd "$ROOT/golden/dart" && "$DART_BIN" test "${args[@]}" -r failures-only)
}

mkdir -p "$BUILD"
run_step "manifest up to date" check_manifest
run_step "rust" rust_tests
run_step "c++" cpp_tests "$BUILD/cpp"
if [[ $ASAN == 1 ]]; then
    run_step "c++ asan/ubsan" cpp_tests "$BUILD/cpp-asan" -DBITIS_SANITIZE=ON
fi
run_step "python" python_tests
run_step "dart setup (pub get, manifest embedded, analyze)" dart_setup
run_step "dart vm" dart_tests vm
run_step "dart chrome/dart2js" dart_tests chrome dart2js
run_step "dart chrome/dart2wasm" dart_tests chrome dart2wasm

echo
echo "=== summary"
status=0
for r in "${RESULTS[@]}"; do
    echo "$r"
    [[ $r == FAIL* ]] && status=1
done
exit $status
