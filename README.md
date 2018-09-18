# Nearby

[![Crates.io](https://img.shields.io/crates/v/nearby.svg)](https://crates.io/crates/nearby)
[![Build Status](https://travis-ci.org/wisespace-io/nearby.png?branch=master)](https://travis-ci.org/wisespace-io/nearby)
[![MIT licensed](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE-MIT)
[![Apache-2.0 licensed](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](./LICENSE-APACHE)

## Description

Scans nearby wifi networks and the devices connected to each network.

## Planned features

- [ ] Map nearby devices ([Netjson for visualization](https://github.com/netjson/netjsongraph.js))
- [ ] Count the number of people around you
- [ ] GeoLocation (Distance from/to a device)

## Build

On Debian based Linux, install `apt-get install libpcap-dev`, so build the project:

```rust
cargo build --release
```

## Usage

Root access is required to `nearby` be able to set the wireless interface on `Monitor Mode`

```rust
sudo target/release/nearby -i your_wireless_adapter
```

I.e: wlan0, or just run iwconfig to get it

## WiFi adapter should support monitor mode

There are many USB WiFi adapters that support monitor mode, i.e:

- Alfa AWUS036NHA
- TP-Link TL-WN722N (ONLY Version 1)
