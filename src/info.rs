use bytes::{Buf, Bytes};
use std::io::Cursor;

pub trait Info {
    fn from_bytes(input: &[u8]) -> Self
    where
        Self: Sized;
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Beacon {
    pub timestamp: u64,
    pub interval: u16,
    pub cap_info: u16,
    pub ssid: String
}

impl Info for Beacon {
    fn from_bytes(input: &[u8]) -> Beacon {
        let mut cursor = Cursor::new(input);

        let timestamp = cursor.get_u64_le();
        let interval = cursor.get_u16_le();
        let cap_info = cursor.get_u16_le();;

        // Parse SSID
        let _element_id = cursor.get_u8();
        let ssid_len = cursor.get_u8();
        let mut buf = Bytes::from(cursor.bytes());
        let ssid =  buf.split_to(ssid_len as usize);

        Beacon {
           timestamp: timestamp,
           interval: interval,
           cap_info: cap_info,
           ssid: String::from_utf8(ssid.to_vec()).unwrap()
        }
    }
}