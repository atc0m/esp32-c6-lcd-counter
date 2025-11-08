# ESP32-C6 LCD with Rust

A simple counter that displays ever-increasing numbers on a 1.47" LCD screen using seven-segment style digits.

## Hardware

ESP32-C6-LCD-1.47 board from Waveshare:
- A little 1.47" TFT display (172x320 pixels)
- An RGB LED that I can make any color (currently using it as a status indicator)
- RISC-V processor

Details from the manufacturer [here](https://www.waveshare.com/wiki/ESP32-C6-LCD-1.47)

## Setup

### 1. Toolchain Installation
Follow the official guide: https://docs.espressif.com/projects/rust/book/getting-started/toolchain.html

- Install Rust: https://rustup.rs/
- RISC-V toolchain
    `rustup toolchain install stable --component rust-src`
- Target (ESP32-C2 and ESP32-C3 vs ESP32-C6 and ESP32-H2)
    `rustup target add riscv32imac-unknown-none-elf`

This project is for the C6 board which is not based on the Xtensa architecture so espup is not needed.

### 2. Useful Tooling
Guide [here](https://docs.espressif.com/projects/rust/book/getting-started/tooling/index.html)

- esp-generate: `cargo install esp-generate  --locked` (turned out this is not needed because I'm using cargo-generate instead?)
- espflash: `cargo install espflash --locked`
- probe-rs: Installed with instruction [here](https://probe.rs/docs/getting-started/installation/)


## Zero to Hero

### `std` vs `no_std`

The Rust on ESP ecosystem offers two primary development paths: a `std`-enabled approach leveraging the Espressif IoT Development Framework (ESP-IDF), and a `no_std` or "bare-metal" approach using the esp-hal crates.

"bare-metal" it out of scope for this project, so I chose the `std`-enabled approach.
Repo setup with `esp-idf-template` project generator.

### Prerequisites for `esp-idf-template`

https://github.com/esp-rs/esp-idf-template#prerequisites

```
cargo install cargo-generate
cargo install ldproxy
```

(espflash was installed in the previous step)

### Generating the Project Template

`cargo generate esp-rs/esp-idf-template cargo`

### First Build and Test

```
cargo build
espflash flash target/riscv32imac-esp-espidf/debug/esp32-c6-lcd-counter
```

## Issues Flashing

The ESP32-C6 board that I have came with the ESP-IDF v5.4.1 bootloader. However the current crates i'm using (as of 2025/10/26) only support ESP-IDF 5.3.3

A github issue with details: https://github.com/esp-rs/esp-idf-template/issues/277

The fix was updating .cargo/config.toml ESP_IDF_VERSION = "v5.3.3" to "v5.4.1"

(also added ESP_IDF_TOOLS_INSTALL_DIR = "global" in config.toml and env `export IDF_MAINTAINER=y`. will confirm soon if this is actually needed)


## Device Features & Pin Connections

### RGB LED

The board comes with WS2812B RGB LED addressable via GPIO pin 8.

A WS2812B LED cannot be controlled by simply setting a GPIO pin to high or low. Requires high-speed, timed serial protocol to set its colour and brightness. The RMT peripheral can be used for this task.

The rust driver library to control the LED: https://github.com/cat-in-136/ws2812-esp32-rmt-driver

### 1.47" TFT Display (SPI Interface)

Creds to AndroidCrypto and his [Medium article](https://medium.com/@androidcrypto/getting-started-with-an-esp32-c6-waveshare-lcd-device-with-1-47-inch-st7789-tft-display-07804fdc589a)

```
Wiring of the ST7789 TFT with an ESP32-C6 Waveshare
TFT   ESP32-C6
GND   GND
VDD   3.3V 
SCL   7
SDA   6 (= "MOSI")
RST   21
DC    15
CS    14
BLK   22
```

## What Does It Actually Do?

When you fire up this bad boy:
1. **Red LED** - "Setup"
2. Display initializes
3. **Green LED** - "Ready"
4. Eight glorious seven-segment digits start counting up from 0
5. The numbers increment every second until reset or it hits 99,999,999 (which would take about 3 years)

The digits are drawn using good old rectangles to create that retro seven-segment display look.

![Image](https://github.com/user-attachments/assets/d90b5822-45c3-499f-8db9-a0087ef0c740)

*Built with Rust 🦀, and trial and error*
