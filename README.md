# esp32cam-rs
<a href="https://github.com/Kezii/esp32cam_rs/actions"><img alt="actions" src="https://github.com/Kezii/esp32cam_rs/actions/workflows/rust.yml/badge.svg"></a>

Rust esp32-cam examples

## Setup
### Git
```bash
git clone <URL> && git submodule update --init
```
or 
```bash
git clone --recursive <URL>
```
### Rust
See
https://docs.esp-rs.org/book/installation/index.html

This project is `std`, so follow the section of the guide for installing `std` dependencies.

## Examples
### SD Card
Saves image, saves to SD card, and sleeps.
Can set sleep time from `CONFIG.TXT` file on the SD card.
`cargo run -r --example sd`

## Credits:
https://github.com/esp-rs/std-training

https://github.com/jlocash/esp-camera-rs
