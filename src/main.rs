extern crate bytes;
extern crate pcap;
extern crate byteorder;
extern crate radiotap;
#[macro_use] 
extern crate error_chain;

mod util;
mod info;
mod errors;
mod dot11;
mod linux_device_management;

use dot11::*;
use bytes::{Buf};
use std::io::Cursor;
use linux_device_management::NetworkInterface;

fn main() {
    let device = "wlp2s0".to_owned();
    let wifi = NetworkInterface::new("wlp2s0").unwrap();

    if let Ok(_value) = wifi.monitor_mode_on() {
        wifi.find_monitor_interfaces().unwrap();

        let mut cap = pcap::Capture::from_device(&device[..])
            .unwrap()
            .timeout(1)
            .rfmon(true)
            .open()
            .unwrap();
        cap.set_datalink(pcap::Linktype(127)).unwrap(); // DLT_IEEE802_11_RADIO = 127

        let mut count = 0;

        // Print out the first 1000 Radiotap headers
        while count < 1000 {
            match cap.next() {
                Ok(packet) => {
                    let data: &[u8] = &packet;

                    let radiotap_header = radiotap::Radiotap::from_bytes(&packet);

                    if radiotap_header.is_ok() {
                        if let Ok(tap_data) = radiotap_header {
                            let mut buf = Cursor::new(data);
                            buf.advance(tap_data.header.length);
                        
                            let header = &Dot11Header::from_bytes(&buf.bytes()).unwrap();
                            //println!("{:?}", header);
                    }
                    count += 1;
                    }
                }
                // There were no packets on the interface before the timeout
                Err(pcap::Error::TimeoutExpired) => {
                    //println!("TIMEOUT");
                    continue;
                }
                Err(e) => {
                    println!("Unexpected error: {:?}", e);
                    break;
                }            
            }  
        }
    }

    if let Err(error) = wifi.monitor_mode_off() {
        print!("{:?}", error);
    }
}