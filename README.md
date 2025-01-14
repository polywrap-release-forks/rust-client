# Polywrap Rust client

Implementation of a client compatible with the [WRAP Protocol](https://github.com/polywrap/specification) in rust

[![codecov](https://codecov.io/gh/polywrap/rust-client/branch/main/graph/badge.svg?token=Z0VNH4R5UR)](https://codecov.io/gh/polywrap/rust-client)

![Public Release Announcement](https://user-images.githubusercontent.com/5522128/177473887-2689cf25-7937-4620-8ca5-17620729a65d.png)

## Overview


| Feature | |
| -- | -- |
| **Invoke**  | ✅ | 
| Subinvoke | ✅ |
| Interfaces | ✅ | 
| Env Configuration | ✅ |
| Client Config | ✅ |
| Plugin Wrapper | ✅ |
| Wrap Manifest | ✅ |
| **Uri Resolution** | ⚙️ |
| Uri: Filesystem | ✅ |
| Uri: Http | ✅ |
| Uri: IPFS | ❌ |
| Uri: ENS | ❌ |



## Build & Test

Before running tests, cases need to be generated, go to `packages/test-utils` and run `cargo run --bin generate`

Now, go back to the root of repository and run tests from all packages:
```
cargo test
```
