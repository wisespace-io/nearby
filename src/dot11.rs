use util::*;
use errors::*;
use bytes::{Buf, IntoBuf, Bytes};
use std::io::{Cursor, self};
use info::*;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum FrameType {
    Management,
    Control,
    Data,
    Unknown
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum FrameSubType {
    AssoReq,
    AssoResp,
    ReassoReq,
    ReassoResp,
    ProbeReq,
    ProbeResp,
    Beacon,
    Atim,
    Disasso,
    Auth,
    Deauth,
    Unknown
}

#[derive(Clone, Debug)]
pub struct Dot11Header {
    pub frame_control: FrameControl,
    pub duration: [u8; 2],
    pub dst: String,
    pub src: String,
    pub bssid: String,
    pub seq_ctl: [u8; 2]
}

impl Dot11Header {
    pub fn from_bytes(input: &[u8]) -> Result<Dot11Header> {
        use std::io::Read;

        let buf = Bytes::from(input).into_buf();
        let mut reader = buf.reader();

        let mut control = [0; 2];
        reader.read(&mut control)?;
        let frame_control = FrameControl::from_bytes(&control)?;

        let mut duration = [0; 2];
        reader.read(&mut duration)?;

        let mut mac_addresses = [0; 18];
        reader.read(&mut mac_addresses)?;

        let (dst, src, bssid) = Dot11Header::parse_address(frame_control, &mac_addresses);

        let mut seq_ctl = [0; 2];
        reader.read(&mut seq_ctl)?;

        let mut dst2 = vec![];
        io::copy(&mut reader, &mut dst2).unwrap();

        let body = Dot11Header::parse_body(frame_control, &dst2[..]);        

        let header = Dot11Header {
            frame_control: frame_control,
            duration: duration,
            dst: dst,
            src: src,
            bssid: bssid,
            seq_ctl: seq_ctl,
        };
        Ok(header)
    }

    fn parse_address(frame_control: FrameControl, input: &[u8]) -> (String, String, String) {
        let mut dst = String::from("");
        let mut src = String::from("");
        let mut bssid = String::from("");

        let addresses = FrameAddresses::from_bytes(input).unwrap();

        if frame_control.to_ds && frame_control.from_ds {
            dst.push_str(&addresses.addr3.addr);
            src.push_str(&addresses.addr4.addr);
        } else if frame_control.to_ds {
            dst.push_str(&addresses.addr2.addr);
            src.push_str(&addresses.addr3.addr);
            bssid.push_str(&addresses.addr1.addr);
        } else if frame_control.from_ds {
            dst.push_str(&addresses.addr3.addr);
            src.push_str(&addresses.addr1.addr);
            bssid.push_str(&addresses.addr2.addr);
        } else {
            dst.push_str(&addresses.addr1.addr);
            src.push_str(&addresses.addr2.addr);
            bssid.push_str(&addresses.addr3.addr);            
        } 

        return (dst, src, bssid)
    }

    fn parse_body(frame_control: FrameControl, input: &[u8]) -> String {
        let dst = String::from("");

        if frame_control.frame_type == FrameType::Management && frame_control.frame_subtype == FrameSubType::Beacon {
            let info = Beacon::from_bytes(input);
            println!("{:?}", info);
        }

        return dst     
    }    
}

#[derive(Copy, Clone, Debug)]
pub struct FrameControl {
    pub frame_type: FrameType,
    pub frame_subtype: FrameSubType,
    pub to_ds: bool,
    pub from_ds: bool,
    pub more_flag: bool,
    pub retry: bool,
    pub pwr_mgmt: bool,
    pub more_data: bool,
    pub wep: bool,
    pub order: bool,
}

impl FrameControl {
    pub fn from_bytes(input: &[u8]) -> Result<FrameControl> {
        let mut cursor = Cursor::new(input);
        let version_type_subtype = cursor.get_u8();
        let flags = cursor.get_u8();

        if FrameControl::protocol_version(version_type_subtype) != 0 {
            bail!("Unknow protocol version");
        }

        let fc = FrameControl {
            frame_type: FrameControl::frame_type(version_type_subtype),
            frame_subtype: FrameControl::frame_subtype(version_type_subtype),
            to_ds: flag_is_set(flags.into(), 0),
            from_ds: flag_is_set(flags.into(), 1),
            more_flag: flag_is_set(flags.into(), 2),
            retry: flag_is_set(flags.into(), 3),
            pwr_mgmt: flag_is_set(flags.into(), 4),
            more_data: flag_is_set(flags.into(), 5),
            wep: flag_is_set(flags.into(), 6),
            order: flag_is_set(flags.into(), 7),
        };

        Ok(fc)
    }

    fn protocol_version(packet: u8) -> u8 {
        packet & 0b0000_0011
    }

    fn frame_type(packet: u8) -> FrameType {
        match (packet & 0b0000_1100) >> 2 {
            0 => FrameType::Management,
            1 => FrameType::Control,
            2 => FrameType::Data,
            _ => FrameType::Unknown
        }
    }

    fn frame_subtype(packet: u8) -> FrameSubType {
        match (packet & 0b1111_0000) >> 4 {
            0 => FrameSubType::AssoReq,
            1 => FrameSubType::AssoResp,
            2 => FrameSubType::ReassoReq,
            3 => FrameSubType::ReassoResp,
            4 => FrameSubType::ProbeReq,
            5 => FrameSubType::ProbeResp,
            8 => FrameSubType::Beacon,
            9 => FrameSubType::Atim,
            10 => FrameSubType::Disasso,
            11 => FrameSubType::Auth,
            12 => FrameSubType::Deauth,
            _ => FrameSubType::Unknown,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FrameAddresses {
    pub addr1: MACField,
    pub addr2: MACField,
    pub addr3: MACField,
    pub addr4: MACField
}

impl FrameAddresses {
    pub fn from_bytes(s: &[u8]) -> Result<FrameAddresses> {
        use std::io::Read;

        let buf = Bytes::from(s).into_buf();
        let mut reader = buf.reader();

        let mut mac_addr1 = [0; 6];
        reader.read(&mut mac_addr1)?;
        let addr1 = MACField::from_slice(&mac_addr1);

        let mut mac_addr2 = [0; 6];
        reader.read(&mut mac_addr2)?;
        let addr2 = MACField::from_slice(&mac_addr2);

        let mut mac_addr3 = [0; 6];
        reader.read(&mut mac_addr3)?;
        let addr3 = MACField::from_slice(&mac_addr3);

        let mut seq_ctl = [0; 2];
        reader.read(&mut seq_ctl)?;

        let mut mac_addr4 = [0; 6];
        reader.read(&mut mac_addr4)?;
        let addr4 = MACField::from_slice(&mac_addr4);
        
        Ok(FrameAddresses {
            addr1,
            addr2,
            addr3,
            addr4
        })
    }
}

#[derive(Clone, Debug)]
pub struct MACField {
    pub addr: String
}

impl MACField {
    pub fn from_slice(s: &[u8]) -> MACField {
        let addr = format!("{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}", s[0], s[1], s[2], s[3], s[4], s[5]);
 
        MACField {
            addr
        }
    }
}