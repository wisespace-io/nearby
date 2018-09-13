use errors::*;
use vendors::*;
use dot11::*;
use info::*;
use radiotap::Radiotap;
use std::collections::HashMap;

static BROADCAST: &'static str = "ff:ff:ff:ff:ff:ff";
static UNSPECIFIED: &'static str = "00:00:00:00:00:00";

#[derive(Clone, Debug)]
pub struct AP {
    ssid: String,
    vendor: String,
    signal: i32,
    devices: Vec<Devices>
}

#[derive(Clone, Debug, Default)]
struct Devices {
    pub ssid: String,
    pub vendor: String,
    pub signal: i32,
}

#[derive(Clone, Debug)]
pub struct Mapper {
    vendors: VendorsDB,
    net_map: HashMap<String, AP>
}

impl Mapper {
    pub fn new() -> Result<Mapper> {
        let vendors = VendorsDB::from_file("data/oui.txt")?;

        Ok(Mapper {
            vendors: vendors,
            net_map: HashMap::new()
        })
    }

    pub fn map(&self, _radio_header: Radiotap, dot11_header: Dot11Header) {
        if dot11_header.frame_control.frame_type == FrameType::Management {
            let info = dot11_header.info;
            let mut device = Devices {..Default::default()};

            if let BodyInformation::Beacon(beacon) = info.clone() {
                device.ssid = beacon.ssid.value;
            }

            if let BodyInformation::ProbeResponse(resp) = info.clone() {
                device.ssid = resp.ssid.value;
            }

            if !dot11_header.bssid.contains(BROADCAST) && !dot11_header.bssid.contains(UNSPECIFIED) {
                println!("BSSID: {:?} SSID: {:?}", dot11_header.bssid, device.ssid);
            }
        }
    }
}