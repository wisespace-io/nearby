# Nearby

[![Crates.io](https://img.shields.io/crates/v/nearby.svg)](https://crates.io/crates/nearby)
[![Build Status](https://travis-ci.org/wisespace-io/nearby.png?branch=master)](https://travis-ci.org/wisespace-io/nearby)
[![MIT licensed](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE-MIT)
[![Apache-2.0 licensed](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](./LICENSE-APACHE)

## Description

Scans nearby wifi networks and the devices connected to each network.

## Planned features

- [x] Map nearby devices ([Netjson for visualization](https://github.com/netjson/netjsongraph.js))
- [x] Count the number of people around you
- [x] Stop Scanning with CTRL-C and print all collected results
- [ ] Watch specific Mac Address (Send alert by email)
- [ ] BLE Indoor Positioning
- [ ] GeoLocation

## Build

On Debian based Linux, install `apt-get install libpcap-dev`, so build the project:

```rust
cargo build --release
```

## Usage

### Nearby Devices

Root access is required to `nearby` be able to set the wireless interface on `Monitor Mode`
You can list the network interfaces with `ip link show` on Ubuntu.

```rust
sudo target/release/nearby -i your_wireless_adapter
```

I.e: wlan0, or just run iwconfig to get it

Use `--netjson` to generate `networks.json` and use it as input to visualization

```rust
sudo target/release/nearby -i your_wireless_adapter --netjson
```

Use `--graph` to start a webserver and visualize the generated file (`networks.json`)

```rust
target/release/nearby --graph
```

### People around you

Use `--people` to generate `people.json`. It will watch Probe Requests and filter the mobiles according to a specified mobile phone vendor list.

```rust
sudo target/release/nearby -i your_wireless_adapter --people
```

Note: The default scan time is 120s, if it stops working after a short period of time often with the error message `libpcap error: The interface went down`, it may be because another running process is causing it. On Ubuntu, you may be the network-manager, try `service network-manager stop`.

## Wifi adapter should support monitor mode

There are many USB Wifi adapters that support monitor mode, i.e:

- Alfa AWUS036NHA
- Alfa AWUS036NEH
- TP-Link TL-WN722N (ONLY Version 1)

## Disclaimer

It is the end user's responsibility to obey all applicable local, state and federal laws. Developers assume no liability and are not responsible for any misuse or damage caused by this program.
