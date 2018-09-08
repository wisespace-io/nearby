use bytes::{Buf, Bytes};
use std::io::Cursor;

pub trait Info {
    fn from_bytes(input: &[u8]) -> Self
        where Self: Sized;
}

#[derive(Clone, Debug)]
pub struct Beacon {
    pub timestamp: u64,
    pub interval: u16,
    pub cap_info: u16,
    pub ssid: SSID,
    pub supported_rates: Vec<f32>
}

impl Info for Beacon {
    fn from_bytes(input: &[u8]) -> Beacon {
        let mut cursor = Cursor::new(input);

        let timestamp = cursor.get_u64_le();
        let interval = cursor.get_u16_le();
        let cap_info = cursor.get_u16_le();;

        let ssid = SSID::from_bytes(cursor.bytes());
        cursor.advance(ssid.ssid_len + 2);

        Beacon {
           timestamp: timestamp,
           interval: interval,
           cap_info: cap_info,
           ssid: ssid,
           supported_rates: supported_rates(cursor.bytes())
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProbeRequest {
    pub ssid: SSID,
    pub supported_rates: Vec<f32>
}

impl Info for ProbeRequest {
    fn from_bytes(input: &[u8]) -> ProbeRequest {
        let mut cursor = Cursor::new(input);

        let ssid = SSID::from_bytes(cursor.bytes());
        cursor.advance(ssid.ssid_len + 2);

        ProbeRequest {
           ssid: ssid,
           supported_rates: supported_rates(cursor.bytes())
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProbeResponse {
    pub timestamp: u64,
    pub interval: u16,
    pub cap_info: u16,
    pub ssid: SSID,
    pub supported_rates: Vec<f32>
}

impl Info for ProbeResponse {
    fn from_bytes(input: &[u8]) -> ProbeResponse {
        let mut cursor = Cursor::new(input);

        let timestamp = cursor.get_u64_le();
        let interval = cursor.get_u16_le();
        let cap_info = cursor.get_u16_le();

        let ssid = SSID::from_bytes(cursor.bytes());
        cursor.advance(ssid.ssid_len + 2);

        ProbeResponse {
           timestamp: timestamp,
           interval: interval,
           cap_info: cap_info,
           ssid: ssid,
           supported_rates: supported_rates(cursor.bytes())
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SSID {
    pub element_id: u8,
    pub ssid_len: usize,
    pub value: String,
}

impl Info for SSID {
    fn from_bytes(input: &[u8]) -> SSID {
        let mut cursor = Cursor::new(input);

        let element_id = cursor.get_u8();
        let ssid_len = cursor.get_u8() as usize;
        let mut buf = Bytes::from(cursor.bytes());
        let ssid =  buf.split_to(ssid_len);

        SSID {
            element_id: element_id,
            ssid_len: ssid_len,
            value: String::from_utf8(ssid.to_vec()).unwrap()
        }
    }
}

pub fn supported_rates(input: &[u8]) -> Vec<f32> {
    let mut rates: Vec<f32> = Vec::new();
    let mut cursor = Cursor::new(input);

    let _element_id = cursor.get_u8();
    let number_of_rates = cursor.get_u8();

    for _x in 0..number_of_rates {        
        let rate = cursor.get_u8();

        match rate {
            0x82 => rates.push(1.0),
            0x84 => rates.push(2.0),
            0x8b => rates.push(5.5),
            0x0c => rates.push(6.0),
            0x12 => rates.push(9.0),
            0x96 => rates.push(11.0),
            0x18 => rates.push(12.0),
            0x24 => rates.push(18.0),
            0x2c => rates.push(22.0),
            0x30 => rates.push(24.0),
            0x42 => rates.push(33.0),
            0x48 => rates.push(36.0),
            0x60 => rates.push(48.0),
            0x6c => rates.push(54.0),
            _ => continue,
        }
    }

    rates
}