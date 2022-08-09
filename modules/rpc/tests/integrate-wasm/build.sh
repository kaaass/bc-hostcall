#!/usr/bin/env sh

# 切换至脚本所在目录
base_dir=$(dirname $0)
cd "${base_dir}" || exit 1

# 构建 guest 程序
cargo build --target wasm32-wasi
