use std::fs::File;
use std::io::prelude::*;
use mapper::*;
use errors::*;
use serde_json::to_string_pretty;

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

pub fn create_netjson(mapper: Mapper) -> Result<(String)> {
    // Print Access Point information
    let mut net: Vec<Collection> = Vec::new();
    for ap in mapper.net_map.values() {
        net.push(ap.clone());
    }

    let net_col = NetworkCollection {
        id: "NetworkCollection".into(),
        collection: net
    };

    let netjson = to_string_pretty(&net_col)?;

    Ok(netjson)
}

pub fn save_netjson(file: &str, content: String) -> Result<()> {
    let mut file = File::create("static/".to_owned() + file)?;
    file.write_all(content.as_bytes())?;
    println!("networks.json generated");
    Ok(())
}