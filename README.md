# Nearby

[![Crates.io](https://img.shields.io/crates/v/nearby.svg)](https://crates.io/crates/nearby)
[![Build Status](https://travis-ci.org/wisespace-io/nearby.png?branch=master)](https://travis-ci.org/wisespace-io/nearby)
[![MIT licensed](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE-MIT)
[![Apache-2.0 licensed](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](./LICENSE-APACHE)

## Description

Scans all nearby wifi networks and the devices connected to each network.

## WORK IN PROGRESS

### Planned features

- [ ] Map all devices nearby (Netjson)
- [ ] Count the number of people around you
- [ ] GeoLocation (Distance from/to a device)

## Build

```rust
cargo build --release
```

## Usage

```rust
nearby -i your_wireless_device
```

I.e: wlan0, or just run iwconfig to get it

## WiFi adapter should support monitor mode

There are many USB WiFi adapters that support monitor mode, i.e:

- Alfa AWUS036NHA
- TP-Link TL-WN722N (ONLY Version 1)
