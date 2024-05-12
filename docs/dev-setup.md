# Rust

```shell

```

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

For coverage test, we use llvm-cov. llvm-cov have a bug that mistakes `bodhi` file inside `resources/bin/bodhi` to be the executable that needs to be profiled.

Till this bug is fixed, we need to rename this bash script to something other than bodhi, run the coverage, and reset the bash script back to original name.

```shell
# pwd $PROJECT_DIR/app/bodhi
mv resources/bin/bodhi{,cli}
cargo llvm-cov clean
cargo llvm-cov nextest --html
mv resources/bin/bodhi{cli,}
```
