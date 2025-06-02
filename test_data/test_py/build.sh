#! /bin/bash

. ./.venv/bin/activate

cd bitis_msgs

cargo clean

../../../target/debug/bitis -i ../test_simple_msg.bitis -d compile -o . -l python

cd -
