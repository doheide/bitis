#! /bin/bash

BITIS_EXE=../../target/debug/bitis
#$BITIS_EXE --help
$BITIS_EXE -i simple_msg.bitis compile -l rust -o rust/src
