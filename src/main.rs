#[macro_use] extern crate error_chain;

mod errors;
mod linux_device_management;

use linux_device_management::NetworkInterface;

fn main() {
    let wifi = NetworkInterface::new("wlp2s0").unwrap();
    wifi.monitor_mode_on();
    wifi.find_monitor_interfaces();
}
