# First Setup for ESP32-C6 development with Rust

This project is based on the ESP32-C6-LCD-1.47 board from waveshare. Details from the manufacturer [here](https://www.waveshare.com/wiki/ESP32-C6-LCD-1.47)

1. Toolchain Installation: https://docs.espressif.com/projects/rust/book/getting-started/toolchain.html

    - Install Rust: https://rustup.rs/
    - RISC-V toolchain
        `rustup toolchain install stable --component rust-src`
    - Target (ESP32-C2 and ESP32-C3 vs ESP32-C6 and ESP32-H2)
        `rustup target add riscv32imac-unknown-none-elf`

    This project is for the C6 board which is not based on the Xtensa architecture so espup is not needed.
    

3. Useful Tooling installation
    Guide [here](https://docs.espressif.com/projects/rust/book/getting-started/tooling/index.html)

    - esp-generate: `cargo install esp-generate  --locked` (turned out this is not needed because I'm using cargo-generate instead?)
    - espflash: `cargo install espflash --locked`
    - probe-rs: Installed with instruction [here](https://probe.rs/docs/getting-started/installation/)


# How I started on this project as a layman

## `std` vs `no_std`

The Rust on ESP ecosystem offers two primary development paths: a `std`-enabled approach leveraging the Espressif IoT Development Framework (ESP-IDF), and a `no_std` or "bare-metal" approach using the esp-hal crates.

"bare-metal" sounds scary so for this project, I chose the `std`-enabled approach, facilitated by the `esp-idf-template` project generator.

## Prequisites for `esp-idf-template`

https://github.com/esp-rs/esp-idf-template#prerequisites

```
cargo install cargo-generate
cargo install ldproxy
```

(espflash was installed in the previous step)

## Generating the project template

`cargo generate esp-rs/esp-idf-template cargo`

## First Build and Test

```
cargo build
espflash flash target/riscv32imac-esp-espidf/debug/esp32-c6-lcd-counter
```

## Issues Flashing

The ESP32-C6 board that I have came with the ESP-IDF v5.4.1 bootloader. However the current crates i'm using (as of 2025/10/26) only support ESP-IDF 5.3.3

A github issue with details: https://github.com/esp-rs/esp-idf-template/issues/277

The fix was updating .cargo/config.toml ESP_IDF_VERSION = "v5.3.3" to "v5.4.1"

