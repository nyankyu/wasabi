# wasabi

A UEFI Boot OS Demo Implemented in Rust
(Reference implementation based on "［作って学ぶ］ OSのしくみⅠ")

## Overview

This project is an implementation of a UEFI environment OS (kernel) in Rust, based on the book "［作って学ぶ］ OSのしくみⅠ" by hikalium.

## Reference Book

- [［作って学ぶ］ OSのしくみⅠ](https://gihyo.jp/book/2025/978-4-297-14859-1) (written by hikalium)

## How to Run

```sh
cargo run
```

## Directory Structure

- [`src/main.rs`](src/main.rs): Entry point and UEFI protocol implementation
- [`scripts/launch_qemu.sh`](scripts/launch_qemu.sh): QEMU launch script
- [`third_party/ovmf/`](third_party/ovmf/): OVMF firmware

## License

MIT License
