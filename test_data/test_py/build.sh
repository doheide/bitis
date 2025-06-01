#! /bin/bash

. ./.venv/bin/activate

cd bitis_msgs

cargo clean

../../..

maturin develop

cd -
