#!/bin/bash

../../target/debug/bitis -d -i simple_msg.bitis compile -o ../impl/rust_impl/src/messages.rs -l rust



../../target/debug/bitis -d -i simple_msg.bitis compile -o ../impl/cpp_impl/src/messages.h -l cpp

