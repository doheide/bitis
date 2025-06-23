#!/bin/bash
set -e

test_dir=$(pwd)

# compile messages to code
../../target/debug/bitis -d compile -i nested_msg.bitis -o ../impl/rust_impl/src/messages.rs -l rust

../../target/debug/bitis -d compile -i nested_msg.bitis -o ../impl/cpp_impl/src/messages.h -l cpp

cd ../impl/python_impl/
. ./.venv/bin/activate
../../../target/debug/bitis compile -i $test_dir/nested_msg.bitis --prevent-update-bitis-lib-in-crate -l python -o py_msg/
cd -


# ****
echo -e "\n******\nrust"
src_path=../impl/rust_impl/src/
main_file=main.rs
if [[ -e $src_path/$main_file ]]; then
    echo "$main_file already exists"
    test -L $src_path/$main_file || (>&2 echo "$src_path/$main_file has to be a symbolic link "; exit 1 )
    rm -v $src_path/$main_file
fi

cpath=$(pwd)
cd ../impl/rust_impl/src
ln -s "$cpath/main.rs" main.rs
cd -

# ****
echo -e "\n******\ncpp"
src_path=../impl/cpp_impl/src/
main_file=main.cpp
if [[ -e $src_path/$main_file ]]; then
    echo "$main_file already exists"
    test -L $src_path/$main_file || (>&2 echo "$src_path/$main_file has to be a symbolic link "; exit 1 )
    rm -v $src_path/$main_file
fi
cpath=$(pwd)
cd ../impl/cpp_impl/src
ln -s "$cpath/main.cpp" main.cpp
cd -

# ****
echo -e "\n******\npython"
src_path=../impl/python_impl/
main_file=main.py
if [[ -e $src_path/$main_file ]]; then
    echo "$main_file already exists"
    test -L $src_path/$main_file || (>&2 echo "$src_path/$main_file has to be a symbolic link "; exit 1 )
    rm -v $src_path/$main_file
fi
cpath=$(pwd)
cd ../impl/python_impl/
ln -s "$cpath/main.py" main.py
cd -



# ****
# build
cpath=$(pwd)
cd ../impl/rust_impl/src
cargo build
cd -


cpath=$(pwd)
cd ../impl/cpp_impl/

rm -rf build
cmake -S . -B build
cmake --build build

cd $cpath

# ****
rm -f *.dat

# ****
set +e

../impl/rust_impl/target/debug/rust_impl

../impl/cpp_impl/build/test_cpp

# ****
../impl/rust_impl/target/debug/rust_impl rs
rs_rs_err=$?
../impl/rust_impl/target/debug/rust_impl cpp
rs_cpp_err=$?
../impl/cpp_impl/build/test_cpp cpp
cpp_cpp_err=$?
../impl/cpp_impl/build/test_cpp rs
cpp_rs_err=$?

echo -e "\nrs_rs_err: $rs_rs_err, rs_cpp_err: $rs_cpp_err, cpp_cpp_err: $cpp_cpp_err, cpp_rs_err: $cpp_rs_err"







