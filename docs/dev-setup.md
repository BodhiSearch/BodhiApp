# Rust

[Rust Installation](https://www.rust-lang.org/tools/install)

# Rust Cargo Utilities

```shell
cargo install tauri-cli
export CC=/usr/bin/clang
export AR=/usr/bin/ar
cargo install cargo-binstall
# faster test runner
cargo binstall cargo-nextest --secure
# for llvm coverage
cargo install cargo-llvm-cov --locked
```

# Running Coverage Test

`cargo xtask coverage --dev -p bodhicore`

