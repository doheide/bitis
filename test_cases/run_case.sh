#!/bin/bash
# Runs one cross-language test case (older framework, see golden/README.md for the
# golden vectors that replace it):
#   test_cases/run_case.sh test_cases/t_oo_msg      (or ./build.sh inside a t_* directory)
#
# 1. compiles <case>/*.bitis to impl/rust_impl/src/messages.rs and impl/cpp_impl/src/messages.h
#    (and the python lib if <case>/main.py exists, impl/python_impl/.venv exists and maturin
#    is installed; otherwise python is skipped),
# 2. links <case>/main.{rs,cpp,py} into impl/ (relative symlinks),
# 3. builds the Rust and C++ programs, writes <case>/*.{rs,cpp,py}.dat and lets every
#    language decode the data of every other language (committed *.py.dat files are
#    decoded by Rust and C++ also when python is skipped).
#
# Environment: BITIS (compiler binary; default: cargo run of this repository),
#              CARGO_FLAGS (e.g. --offline), CPP_BUILD_DIR (default: impl/cpp_impl/build).
set -e

[[ $# == 1 && -d $1 ]] || { echo "usage: $0 <test case directory>"; exit 2; }
case_dir=$(cd "$1" && pwd)
name=$(basename "$case_dir")
tc_root=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
root=$(dirname "$tc_root")
impl=$tc_root/impl
cpp_build=${CPP_BUILD_DIR:-$impl/cpp_impl/build}
CARGO_FLAGS=${CARGO_FLAGS:-}

shopt -s nullglob
bitis_files=("$case_dir"/*.bitis)
[[ ${#bitis_files[@]} == 1 ]] || { echo "expected exactly one .bitis file in $case_dir"; exit 2; }
bitis_file=${bitis_files[0]}

bitis() {
    if [[ -n "${BITIS:-}" ]]; then "$BITIS" "$@"
    # shellcheck disable=SC2086
    else cargo run -q $CARGO_FLAGS --manifest-path "$root/Cargo.toml" --bin bitis -- "$@"; fi
}

with_python=0
if [[ -f $case_dir/main.py && -f $impl/python_impl/.venv/bin/activate ]] && command -v maturin > /dev/null; then
    with_python=1
fi

# ****
echo -e "\n****** compile $bitis_file"
bitis compile -i "$bitis_file" -o "$impl/rust_impl/src/messages.rs" -l rust
bitis compile -i "$bitis_file" -o "$impl/cpp_impl/src/messages.h" -l cpp
ln -sfn "../../../$name/main.rs" "$impl/rust_impl/src/main.rs"
ln -sfn "../../../$name/main.cpp" "$impl/cpp_impl/src/main.cpp"
if [[ $with_python == 1 ]]; then
    ln -sfn "../../$name/main.py" "$impl/python_impl/main.py"
    # shellcheck disable=SC1091
    (cd "$impl/python_impl" && . ./.venv/bin/activate &&
        bitis compile -i "$bitis_file" --prevent-update-bitis-lib-in-crate -l python -o py_msg/)
else
    echo "* python skipped (needs $name/main.py, impl/python_impl/.venv and maturin)"
fi

# ****
echo -e "\n****** build"
# shellcheck disable=SC2086
cargo build $CARGO_FLAGS --manifest-path "$impl/rust_impl/Cargo.toml"
rm -rf "$cpp_build"
cmake -S "$impl/cpp_impl" -B "$cpp_build" > /dev/null
cmake --build "$cpp_build"

rust_impl() {
    # shellcheck disable=SC2086
    cargo run -q $CARGO_FLAGS --manifest-path "$impl/rust_impl/Cargo.toml" -- "$@"
}
cpp_impl() { "$cpp_build/test_cpp" "$@"; }
py_impl() {
    # shellcheck disable=SC1091
    (. "$impl/python_impl/.venv/bin/activate" && python "$impl/python_impl/main.py" "$@")
}

# ****
cd "$case_dir"
rm -f ./*.rs.dat ./*.cpp.dat
[[ $with_python == 1 ]] && rm -f ./*.py.dat
echo -e "\n****** write"
rust_impl
cpp_impl
[[ $with_python == 1 ]] && py_impl

echo -e "\n****** cross decode"
set +e
readers=(rs cpp)
writers=(rs cpp)
[[ $with_python == 1 ]] && readers+=(py)
# data written by python earlier (committed *.py.dat) is decoded also without python
py_dat=(./*.py.dat)
[[ ${#py_dat[@]} -gt 0 ]] && writers+=(py)
summary=""
failed=0
for reader in "${readers[@]}"; do
    for writer in "${writers[@]}"; do
        case $reader in
            rs) rust_impl "$writer" ;;
            cpp) cpp_impl "$writer" ;;
            py) py_impl "$writer" ;;
        esac
        rc=$?
        summary+="${reader}_${writer}_err: $rc, "
        [[ $rc != 0 ]] && failed=1
    done
done
echo -e "\n$name: ${summary%, }"
exit $failed
