# rued
[Ru]st [E]lectronic [D]atalogger - my explorations into an embedded, async, real-world project

This is heavily inspired by [ruwm](https://github.com/ivmarkov/ruwm) by [ivmarkov](https://github.com/ivmarkov).

If you have questions, [find me on Discord](https://discord.gg/rust-lang-community) in the `#embedded` channel or in `#esp-rs` on [Matrix](https://matrix.to/#/#esp-rs:matrix.org)

## Yet another IoT stack?

With any IoT 'stack', there are some rudimentary basics; this repo is an attempt at building these out using the [embedded_svc](https://github.com/esp-rs/embedded-svc) library and its family of crates from Espressif, such as [esp-idf-svc](https://github.com/esp-rs/esp-idf-svc), [esp-idf-sys](https://github.com/esp-rs/esp-idf-sys), and [esp-idf-hal](https://github.com/esp-rs/esp-idf-hal).

### Feature list

| Done? | Feature | Notes |
|-----|---|---|
| [ ] | Wifi | *Currently unstable; needs a reboot on first flash. |
| [ ] | SNTP |  |
| [ ] | Httpd server | Handlers can be customised |
| [ ] | Httpd server: JSON responses | Also implemented for default ESP error, if the request `Content-Type` is also JSON. |
| [ ] | SNTP fallback to RTC |  |
| [ ] | I2C | Added on a per-sensor basis |
| [ ] | Display | Needs testing |
| [ ] | RTOS | Working across multiple threads. |

- [P]: pending.


## Quick setup

- Install rustup, preferrably in linux.  Using an x86 CPU will help a great deal. Attempts on an [Apple M1 Max failed terribly](https://desilva.io/posts/rust-for-embedded-is-better-on-x86) for me.
- Build your ESP toolchain by following: https://github.com/esp-rs/rust-build
- Run `cargo +esp build --release`
- Use `espflash` to flash your ESP32, make sure you enable the `--monitor` option to watch the output.

Conecting to Wifi can be flakey at times, I usually have to reboot it once after each flashing attempt to connect.

## Features


## Httpd Server
