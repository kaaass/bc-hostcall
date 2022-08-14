# Benchmark

场景：Host 发起调用，Wasm 在调用中发起 Hostcall 调用 `http_get`。

测试前需要在本地 80 端口开启任意 Web 服务。

## 测试结果

```
5,10
// wit
Test http://127.0.0.1/~kaaass/512B.txt: 140ms
Test http://127.0.0.1/~kaaass/1K.txt: 96ms
Test http://127.0.0.1/~kaaass/5K.txt: 107ms
Test http://127.0.0.1/~kaaass/10K.txt: 131ms
Test http://127.0.0.1/~kaaass/100K.txt: 196ms
Test http://127.0.0.1/~kaaass/1M.txt: 936ms
Test http://127.0.0.1/~kaaass/10M.txt: 7836ms

// our
Test http://127.0.0.1/~kaaass/512B.txt: 22ms
Test http://127.0.0.1/~kaaass/1K.txt: 20ms
Test http://127.0.0.1/~kaaass/5K.txt: 27ms
Test http://127.0.0.1/~kaaass/10K.txt: 20ms
Test http://127.0.0.1/~kaaass/100K.txt: 20ms
Test http://127.0.0.1/~kaaass/1M.txt: 41ms
Test http://127.0.0.1/~kaaass/10M.txt: 300ms

5,50
// wit
Test http://127.0.0.1/~kaaass/512B.txt: 517ms
Test http://127.0.0.1/~kaaass/1K.txt: 375ms
Test http://127.0.0.1/~kaaass/5K.txt: 415ms
Test http://127.0.0.1/~kaaass/10K.txt: 403ms
Test http://127.0.0.1/~kaaass/100K.txt: 704ms
Test http://127.0.0.1/~kaaass/1M.txt: 4392ms
Test http://127.0.0.1/~kaaass/10M.txt: 38363ms

// our
Test http://127.0.0.1/~kaaass/512B.txt: 83ms
Test http://127.0.0.1/~kaaass/1K.txt: 71ms
Test http://127.0.0.1/~kaaass/5K.txt: 71ms
Test http://127.0.0.1/~kaaass/10K.txt: 73ms
Test http://127.0.0.1/~kaaass/100K.txt: 82ms
Test http://127.0.0.1/~kaaass/1M.txt: 176ms
Test http://127.0.0.1/~kaaass/10M.txt: 1279ms

5,100
// wit
Test http://127.0.0.1/~kaaass/512B.txt: 815ms
Test http://127.0.0.1/~kaaass/1K.txt: 708ms
Test http://127.0.0.1/~kaaass/5K.txt: 755ms
Test http://127.0.0.1/~kaaass/10K.txt: 841ms
Test http://127.0.0.1/~kaaass/100K.txt: 1411ms
Test http://127.0.0.1/~kaaass/1M.txt: 8816ms
Test http://127.0.0.1/~kaaass/10M.txt: 76419ms

// our
Test http://127.0.0.1/~kaaass/512B.txt: 244ms
Test http://127.0.0.1/~kaaass/1K.txt: 135ms
Test http://127.0.0.1/~kaaass/5K.txt: 137ms
Test http://127.0.0.1/~kaaass/10K.txt: 138ms
Test http://127.0.0.1/~kaaass/100K.txt: 145ms
Test http://127.0.0.1/~kaaass/1M.txt: 325ms
Test http://127.0.0.1/~kaaass/10M.txt: 2368ms
```