use std::fs::File;
use std::io::prelude::*;
use crate::mapper::*;
use crate::errors::*;
use serde_json::to_string_pretty;
use serde::{Serialize};

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WifiAccessPoints {
    wifi_access_points: Vec<Macs>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Macs {
    mac_address: String,
    signal_strength: i8,
}

#[inline]
pub fn flag_is_set(data: u8, bit: u8) -> bool {
    if bit == 0 {
        let mask = 1;
        (data & mask) > 0
    } else {
        let mask = 1 << bit;
        (data & mask) > 0
    }
}

pub fn create_netjson(mapper: Mapper) -> Result<String> {
    // Print Access Point information
    let mut net: Vec<Collection> = Vec::new();
    for ap in mapper.net_map.values() {
        net.push(ap.clone());
    }

    let net_col = NetworkCollection {
        id: "NetworkCollection".into(),
        collection: net,
    };

    let netjson = to_string_pretty(&net_col)?;

    Ok(netjson)
}

pub fn format_people_json(mapper: Mapper) -> Result<String> {
    let mut people_vec: Vec<People> = Vec::new();

    for person in mapper.people_map.values() {
        people_vec.push(person.clone());
    }

    let json = to_string_pretty(&people_vec)?;

    Ok(json)
}

pub fn save_netjson(file: &str, content: String) -> Result<()> {
    let mut file = File::create("static/".to_owned() + file)?;
    file.write_all(content.as_bytes())?;
    println!("static/networks.json generated");
    Ok(())
}
