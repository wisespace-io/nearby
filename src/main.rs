#[macro_use]
extern crate serde_derive;
use pcap;
use radiotap;

#[macro_use]
extern crate error_chain;

mod util;
mod errors;
mod dot11;
mod server;
mod mapper;
mod linux_device_management;

use crate::errors::*;
use crate::dot11::header::*;
use crate::mapper::*;
use bytes::{Buf};
use std::io::Cursor;
use clap::{Arg, App};
use std::time::Instant;
use console::{style, Emoji, Term};
use crate::linux_device_management::NetworkInterface;

const TIMEOUT: i32 = 10;
const DEFAULT_EXECUTION_WINDOW: usize = 5;
const LONG_EXECUTION_WINDOW: usize = 10;

fn main() -> Result<()> {
    let matches = App::new("Nearby")
                        .args(&[
                            Arg::with_name("interface")
                                    .takes_value(true)
                                    .short("i")
                                    .long("interface")
                                    .multiple(true)
                                    .help("wireless interface")
                                    .required(false),
                            Arg::with_name("graph")
                                    .help("Visualize the netjson")
                                    .short("g")
                                    .long("graph")
                                    .required(false),
                            Arg::with_name("netjson")
                                    .help("Create a netjson file")
                                    .short("n")
                                    .long("netjson")
                                    .required(false),
                            Arg::with_name("people")
                                    .help("Outputs a json with the devices")
                                    .short("p")
                                    .long("people")
                                    .required(false)                                
                        ]).get_matches();
    
    if let Some(device) = matches.value_of("interface") {
        let mut wifi = NetworkInterface::new(device)?;

        if let Ok(_value) = wifi.monitor_mode_on() {
            wifi.find_monitor_interfaces()?;
            wifi.find_supported_channels()?;

            let capture = pcap::Capture::from_device(&device[..])?;
            let mut cap = match capture.timeout(TIMEOUT).rfmon(true).open() {
                Ok(cap) => cap,
                Err(_e) => bail!("Can not open device, you need root access"),
            };

            let people = matches.is_present("people");
            let mut execution_window = DEFAULT_EXECUTION_WINDOW * wifi.channels.len();

            if people {
                execution_window = LONG_EXECUTION_WINDOW * wifi.channels.len();
            }
            // DLT_IEEE802_11_RADIO = 127
            if let Ok(_result) = cap.set_datalink(pcap::Linktype(127)) {
                let mut mapper = Mapper::new()?;
                let term = Term::stdout();
                let start = Instant::now();

                wifi.start_channel_switch();
                while start.elapsed().as_secs() < (execution_window as u64) {
                    let remaining = (execution_window as u64) - start.elapsed().as_secs();
                    term.write_line(&format!("{} Searching devices: remaining {} sec. Use CTRL-C to stop ", Emoji("📶", "📡 "), style(remaining).red()))?;
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
                                    if let Some(ap) = mapper.map(tap_data, dot11_header, people) {
                                        term.write_line(&format!("Access point {} signal {} current channel {} {}", 
                                                        style(ap.ssid).cyan(),
                                                        style(ap.signal).cyan(),
                                                        style(ap.current_channel).cyan(),
                                                        "                      "))?;
                                    }                                
                                }
                            }
                        }
                        // There were no packets on the interface before the timeout
                        Err(pcap::Error::TimeoutExpired) => {
                            continue;
                        }
                        Err(e) => {
                            bail!("Unexpect error: {}", e.to_string())
                        }
                    }

                    if !wifi.running() {
                        println!("");
                        println!("=>>>>>> Process stopped by user (CTRL-C)");
                        break;                        
                    }
                }

                term.clear_line()?;

                if people {
                    println!("{}", util::format_people_json(mapper)?);                  
                } else {
                    let netjson = util::create_netjson(mapper)?;
                    if matches.is_present("netjson") {
                        let output = matches.value_of("netjson").unwrap_or("networks.json");
                        util::save_netjson(output, netjson)?;
                    } else {
                        println!("{}", netjson);
                    }
                }
            } else {
                bail!("Can not set datalink")
            }
        }

        if let Err(e) = wifi.monitor_mode_off() {
            bail!("Monitor Mode Off: {:?}", e.to_string())
        }
    }

    if matches.is_present("graph") {
        server::start();
    }

    Ok(())
}
