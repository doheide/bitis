#!/bin/bash
set -e

test_dir=$(pwd)


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


# compile messages to code
../../target/debug/bitis -d compile -i oo_msg.bitis -o ../impl/rust_impl/src/messages.rs -l rust

../../target/debug/bitis -d compile -i oo_msg.bitis -o ../impl/cpp_impl/src/messages.h -l cpp

cd ../impl/python_impl/
. ./.venv/bin/activate
../../../target/debug/bitis compile -i $test_dir/oo_msg.bitis --prevent-update-bitis-lib-in-crate -l python -o py_msg/
cd -


# ****
# build
echo -e "\n******\nrust build"
cpath=$(pwd)
cd ../impl/rust_impl/
cargo build
cd -

echo -e "\n******\ncpp build"
cpath=$(pwd)
cd ../impl/cpp_impl/

rm -rf build
cmake -S . -B build
cmake --build build

cd $cpath

# ****
rm -f *.dat

# ****
../impl/rust_impl/target/debug/rust_impl
../impl/cpp_impl/build/test_cpp
python ../impl/python_impl/main.py


# ****
../impl/rust_impl/target/debug/rust_impl rs
rs_rs_err=$?
../impl/rust_impl/target/debug/rust_impl cpp
rs_cpp_err=$?
../impl/rust_impl/target/debug/rust_impl py
rs_py_err=$?
../impl/cpp_impl/build/test_cpp cpp
cpp_cpp_err=$?
../impl/cpp_impl/build/test_cpp rs
cpp_rs_err=$?
../impl/cpp_impl/build/test_cpp py
cpp_py_err=$?

python ../impl/python_impl/main.py py
py_py_err=$?
python ../impl/python_impl/main.py rs
py_rs_err=$?
python ../impl/python_impl/main.py cpp
py_cpp_err=$?


echo -e "\nrs_rs_err: $rs_rs_err, rs_cpp_err: $rs_cpp_err, cpp_cpp_err: $cpp_cpp_err, cpp_rs_err: $cpp_rs_err, cpp_py_err: $cpp_py_err, rs_py_err: $rs_py_err, py_py_err: $py_py_err, py_rs_err: $py_rs_err, py_cpp_err: $py_cpp_err"





