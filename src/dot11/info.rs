use bytes::{Buf, Bytes};
use std::io::Cursor;

#[derive(Clone, Debug)]
pub enum BodyInformation {
    Beacon(Beacon),
    ProbeRequest(ProbeRequest),
    ProbeResponse(ProbeResponse),
    AssociationRequest(AssociationRequest),
    AssociationResponse(AssociationResponse),
    UnHandled(bool)
}

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
    pub supported_rates: Vec<f32>,
    pub country: Country
}

impl Info for Beacon {
    fn from_bytes(input: &[u8]) -> Beacon {
        let mut cursor = Cursor::new(input);

        let timestamp = cursor.get_u64_le();
        let interval = cursor.get_u16_le();
        let cap_info = cursor.get_u16_le();

        let ssid = SSID::from_bytes(cursor.bytes());
        cursor.advance(ssid.ssid_len + 2); // 2 accounts for Id + Len
        let supported_rates = supported_rates(cursor.bytes());
        cursor.advance(supported_rates.len() + 2); // 2 accounts for Id + Len
        let country = get_country(cursor.bytes());

        Beacon {
           timestamp: timestamp,
           interval: interval,
           cap_info: cap_info,
           ssid: ssid,
           supported_rates: supported_rates,
           country: country
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
    pub supported_rates: Vec<f32>,
    pub country: Country
}

impl Info for ProbeResponse {
    fn from_bytes(input: &[u8]) -> ProbeResponse {
        let mut cursor = Cursor::new(input);

        let timestamp = cursor.get_u64_le();
        let interval = cursor.get_u16_le();
        let cap_info = cursor.get_u16_le();

        let ssid = SSID::from_bytes(cursor.bytes());
        cursor.advance(ssid.ssid_len + 2); // 2 accounts for Id + Len
        let supported_rates = supported_rates(cursor.bytes());
        cursor.advance(supported_rates.len() + 2); // 2 accounts for Id + Len
        let country = get_country(cursor.bytes());

        ProbeResponse {
           timestamp: timestamp,
           interval: interval,
           cap_info: cap_info,
           ssid: ssid,
           supported_rates: supported_rates,
           country: country
        }
    }
}

#[derive(Clone, Debug)]
pub struct AssociationRequest {
    pub cap_info: u16,
    pub interval: u16,
    pub ssid: SSID,
    pub supported_rates: Vec<f32>
}

impl Info for AssociationRequest {
    fn from_bytes(input: &[u8]) -> AssociationRequest {
        let mut cursor = Cursor::new(input);

        let cap_info = cursor.get_u16_le();
        let interval = cursor.get_u16_le();
        let ssid = SSID::from_bytes(cursor.bytes());
        cursor.advance(ssid.ssid_len + 2);

        AssociationRequest {
           cap_info: cap_info,
           interval: interval,
           ssid: ssid,
           supported_rates: supported_rates(cursor.bytes())
        }
    }
}

#[derive(Clone, Debug)]
pub struct AssociationResponse {
    pub cap_info: u16,
    pub status_code: u16,
    pub association_id: u16,
    pub supported_rates: Vec<f32>
}

impl Info for AssociationResponse {
    fn from_bytes(input: &[u8]) -> AssociationResponse {
        let mut cursor = Cursor::new(input);

        let cap_info = cursor.get_u16_le();
        let status_code = cursor.get_u16_le();
        let association_id = cursor.get_u16_le();

        AssociationResponse {
           cap_info: cap_info,
           status_code: status_code,
           association_id: association_id,
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
            value: String::from_utf8(ssid.to_vec()).unwrap_or("".to_string())
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Country {
    pub country_code: String,
}

impl Info for Country {
    fn from_bytes(input: &[u8]) -> Country {
        let mut buf = Bytes::from(input);
        let country_code =  buf.split_to(3); // Country code has 3 bytes

        // We should include the supported channels
        Country {
            country_code: String::from_utf8(country_code.to_vec()).unwrap_or("".to_string())
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

pub fn get_country(input: &[u8]) -> Country {
    let mut cursor = Cursor::new(input);
    let mut country = Country { ..Default::default() };

    loop {
        let element_id = cursor.get_u8();
        let len = cursor.get_u8() as usize;

        // Skipping some fields as we just want the country info for now
        match element_id {
            0x02 => cursor.advance(len), // FH Parameter Set
            0x03 => cursor.advance(len), // DS Parameter Set
            0x04 => cursor.advance(len), // CF Parameter Set
            0x05 => cursor.advance(len), // TIM
            0x06 => cursor.advance(len), // IBSS
            0x07 => {
                country = Country::from_bytes(cursor.bytes());
                break;
            },
            0x32...0x42 => cursor.advance(len), // Can appear before country
            _ => {
                break;
            }
        }
    }

    country
}