#!/usr/bin/env sh
# compile_tests.sh - 用于编译测试用例所需工件的脚本
#
# Copyright (C) 2022 KAAAsS

base_dir=$(dirname $0)
cd $base_dir || exit

# low-level 模块
./modules/low-level/tests/guest/build.sh

# rpc 模块
./modules/rpc/tests/integrate-wasm/build.sh

# async-api 模块
./modules/async-api/tests/unittest-future/build.sh
./modules/async-api/tests/integrate-wasm/build.sh

# tests
./tests/wasm-dispatch/build.sh
./tests/wasm-service-a/build.sh
./tests/wasm-service-b/build.sh

# benchmark
./benchmark/wasm/build.sh
./benchmark/wit-wasm/build.sh
