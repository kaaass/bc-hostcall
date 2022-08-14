# 调用示例

## 模块说明

### Host (CLI)

- 导出：`host::http_get`
- 导入：`*::app`

### `wasm-dispatch`

- 导出：`dispatch::app`
- 导入：`service::do_service`

### `wasm-service-a`

- 导出：`service::do_service`
- 导入：`host::http_get`

### `wasm-service-b`

- 导出：`service::do_service`
- 导入：`host::http_get`

## 测试流程

```
# 热加载
load ./tests/wasm-dispatch/wasm-dispatch.wasm
load ./tests/wasm-service-a/wasm-service-a.wasm
# 调用模块、模块间调用、模块调用 Host
call_app dispatch asdasd
# 热更新（service 模块）
load ./tests/wasm-service-b/wasm-service-b.wasm
list
call_app dispatch qweqwe
```

[![asciicast](https://asciinema.org/a/wxhUJ622q35qPSogN2YjM9ihH.svg)](https://asciinema.org/a/wxhUJ622q35qPSogN2YjM9ihH)
