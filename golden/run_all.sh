#!/usr/bin/env bash
# Runs the golden vector tests of all runtimes against golden/vectors/manifest.txt
# (see golden/README.md).
#
#   golden/run_all.sh [--no-asan]
#
# Environment:
#   CARGO_FLAGS                       extra cargo flags, e.g. "--offline"
#   FETCHCONTENT_SOURCE_DIR_GOOGLETEST local googletest checkout for offline C++ builds
#   BITIS_GOLDEN_BUILD_DIR            C++ build directory (default: target/golden)
#   CXX                               C++ compiler (cmake default otherwise)
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
    (cd "$ROOT" && cargo test -q $CARGO_FLAGS --test golden)
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

# --- Dart runtime
dart_tests() {
    # PLACEHOLDER: enable once the Dart code generator (bitis compile -l dart) is merged, e.g.
    #   bitis compile -l dart -i golden/schema/golden.bitis -o golden/gen/dart/golden.dart
    #   (cd golden/dart && dart pub get && dart test)
    echo "SKIP: Dart golden test not implemented yet (see golden/README.md)"
    return 77
}

mkdir -p "$BUILD"
run_step "manifest up to date" check_manifest
run_step "rust" rust_tests
run_step "c++" cpp_tests "$BUILD/cpp"
if [[ $ASAN == 1 ]]; then
    run_step "c++ asan/ubsan" cpp_tests "$BUILD/cpp-asan" -DBITIS_SANITIZE=ON
fi
run_step "python" python_tests
run_step "dart" dart_tests

echo
echo "=== summary"
status=0
for r in "${RESULTS[@]}"; do
    echo "$r"
    [[ $r == FAIL* ]] && status=1
done
exit $status
