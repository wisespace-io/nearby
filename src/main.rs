#[macro_use]
extern crate serde_derive;
extern crate bytes;
extern crate pcap;
extern crate console;
extern crate byteorder;
extern crate radiotap;
#[macro_use] 
extern crate error_chain;
extern crate clap;
extern crate serde;
extern crate serde_json;

mod util;
mod info;
mod errors;
mod dot11;
mod vendors;
mod mapper;
mod linux_device_management;

use errors::*;
use dot11::*;
use mapper::*;
use bytes::{Buf};
use std::io::Cursor;
use clap::{Arg, App};
use std::time::Instant;
use console::{style, Term};
use linux_device_management::NetworkInterface;

fn main() -> Result<()> {
    let matches = App::new("Nearby")
                        .args(&[
                            Arg::with_name("interface")
                                    .takes_value(true)
                                    .short("i")
                                    .long("interface")
                                    .multiple(true)
                                    .help("wireless interface")
                                    .required(true),
                            Arg::with_name("nolog")
                                    .help("Don't output log file")
                                    .long("nolog")
                        ]).get_matches();
    
    if let Some(device) = matches.value_of("interface") {
        let wifi = NetworkInterface::new(device)?;

        if let Ok(_value) = wifi.monitor_mode_on() {
            wifi.find_monitor_interfaces()?;

            let mut cap = pcap::Capture::from_device(&device[..])?;

            let mut cap = match cap.timeout(1).rfmon(true).open() {
                Ok(cap) => cap,
                Err(_e) => bail!("Can not open device, you need root access"),
            };

            // DLT_IEEE802_11_RADIO = 127
            if let Ok(_result) = cap.set_datalink(pcap::Linktype(127)) {
                let mut mapper = Mapper::new()?;
                let term = Term::stdout();
                let start = Instant::now();

                while start.elapsed().as_secs() < 15 {
                    let elapsed = start.elapsed().as_secs();
                    term.write_line(&format!("Searching devices ... elapsed time {}", style(elapsed).cyan()))?;
                    term.move_cursor_up(1)?;
                    match cap.next() {
                        Ok(packet) => {
                            let data: &[u8] = &packet;
                            let radiotap_header = radiotap::Radiotap::from_bytes(&packet);
                            if radiotap_header.is_ok() {
                                if let Ok(tap_data) = radiotap_header {
                                    let mut buf = Cursor::new(data);
                                    buf.advance(tap_data.header.length);
                                
                                    let dot11_header = Dot11Header::from_bytes(&buf.bytes())?;
                                    mapper.map(tap_data, dot11_header);
                                }
                            }
                        }
                        // There were no packets on the interface before the timeout
                        Err(pcap::Error::TimeoutExpired) => {
                            continue;
                        }
                        Err(e) => {
                            bail!("Unexpect error: {:?}", e.to_string())
                        }
                    }
                }

                term.clear_line()?;

                // Print Access Point information
                let mut net: Vec<Collection> = Vec::new();
                for ap in mapper.net_map.values() {
                    net.push(ap.clone());
                }

                let net_col = NetworkCollection {
                    id: "NetworkCollection".into(),
                    collection: net
                };
                let netjson = serde_json::to_string_pretty(&net_col)?;
                println!("{}", netjson);

            } else {
                bail!("Can not set datalink")
            }
        }

        if let Err(e) = wifi.monitor_mode_off() {
            bail!("Monitor Mode Off: {:?}", e.to_string())
        }
    }

    Ok(())
}
