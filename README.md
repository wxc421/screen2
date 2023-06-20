

## build script
```shell
cargo clean && cargo build && cp ./target/debug/screensnap.dll /c/coding/screenshot-test/screensnap.dll
cargo clean && cargo build -r && cp ./target/release/screensnap.dll /c/coding/screenshot-test/screensnap.dll
```