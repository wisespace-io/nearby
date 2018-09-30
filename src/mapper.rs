use errors::*;
use dot11::vendors::*;
use dot11::header::*;
use dot11::info::*;
use radiotap::Radiotap;
use std::collections::HashMap;

const FREE_SPACE_PATH_LOSS: f32 = 27.55;
static PROTOCOL: &'static str = "802.11";
static BROADCAST: &'static str = "ff:ff:ff:ff:ff:ff";
static UNSPECIFIED: &'static str = "00:00:00:00:00:00";
static MULTICAST: &'static str = "33:33:00:";

// Access Point Information mapped to NetJson format
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkCollection {
    #[serde(rename = "type")]
    pub id: String,
    pub collection: Vec<Collection>
}

// Access Point Information mapped to NetJson format
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Collection {
    #[serde(rename = "type")]
    pub ssid: String,
    pub protocol: String,
    pub version: String,
    pub router_id: String, // BSSID
    pub label: String,
    pub signal: i8,
    pub current_channel: u8,
    nodes: Vec<Node>,
    links: Vec<Link>
}

impl Collection {
    fn new() -> Collection {
        Collection {
            ssid: String::new(),
            protocol: PROTOCOL.into(),
            version: String::new(),
            router_id: String::new(),
            label: String::new(),
            signal: 0,
            current_channel: 0,
            nodes: Vec::new(),
            links: Vec::new()
        }
    }

    fn push_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    fn push_link(&mut self, link: Link) {
        self.links.push(link);
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
struct Node {
    #[serde(rename = "id")]
    pub mac: String,
    pub properties: Properties
}

impl Node {
    fn new(mac: String, vendor: String, signal: i8) -> Node {       
        let properties = Properties {
            vendor: vendor,
            signal: signal
        };

        Node {
            mac: mac,
            properties: properties
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
struct Properties {
    pub vendor: String,
    pub signal: i8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
struct Link {
    pub source: String,
    pub target: String
}

impl Link {
    fn new(src: String, dst: String) -> Link {
        Link {
            source: src,
            target: dst
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct People {
    pub mac: String,
    pub vendor: String,
    pub signal: i8,
    pub distance: f32
}

#[derive(Clone, Debug)]
pub struct Mapper {
    pub vendors: VendorsDB,
    pub net_map: HashMap<String, Collection>,
    pub people_map: HashMap<String, People>
}

impl Mapper {
    pub fn new() -> Result<Mapper> {
        let vendors = VendorsDB::from_file("data/oui.txt")?;

        Ok(Mapper {
            vendors: vendors,
            net_map: HashMap::new(),
            people_map: HashMap::new()
        })
    }

    pub fn map(&mut self, radio_header: Radiotap, dot11_header: Dot11Header, people: bool) -> Option<Collection> {
        let mut new_ap = None;
        let info = dot11_header.info.clone();
        let frame_type = dot11_header.frame_control.frame_type;
        let frame_subtype = dot11_header.frame_control.frame_subtype;
        let signal = match radio_header.antenna_signal {
            Some(antenna_signal) => antenna_signal.value,
            None => 0
        };

        let freq: f32 = match radio_header.channel {
            Some(channel) => channel.freq as f32,
            None => 0.0
        };

        if people {
            if frame_type == FrameType::Management && frame_subtype == FrameSubType::ProbeReq {
                self.add_people(dot11_header.src, freq, signal);
            }
        } else {
            // We should monitor the Probe Request frames for positioning information
            if frame_type == FrameType::Management && frame_subtype == FrameSubType::ProbeReq {
                self.add_to_collection(dot11_header.src, dot11_header.bssid, signal);
            } else if frame_type == FrameType::Data {
                if frame_subtype == FrameSubType::QoS || frame_subtype == FrameSubType::Data {
                    self.add_to_collection(dot11_header.src, dot11_header.bssid.clone(), signal);
                    self.add_to_collection(dot11_header.dst, dot11_header.bssid, signal);
                } else if frame_subtype == FrameSubType::NullData {
                    // NullData informs Device Power Serving mode
                    self.add_to_collection(dot11_header.dst, dot11_header.bssid, signal);
                }
            } else if frame_type == FrameType::Management {
                // Lets use the Beacon frame to get Access Point information
                if let BodyInformation::Beacon(beacon) = info.clone() {
                    new_ap = self.add_access_point(beacon, signal, dot11_header);
                }
            }
        }
        new_ap
    }

    fn add_access_point(&mut self, beacon: Beacon, signal: i8, dot11_header: Dot11Header) -> Option<Collection> {
        if !dot11_header.bssid.contains(BROADCAST) && !dot11_header.bssid.contains(UNSPECIFIED) {
            let header = dot11_header.clone();
            if !self.net_map.contains_key(&header.bssid.clone()) {
                let mut access_point = Collection::new();

                access_point.ssid = beacon.ssid.value.clone();
                access_point.signal = signal;
                access_point.current_channel = beacon.current_channel;
                access_point.router_id = header.bssid.clone();
                access_point.label = self.vendors.lookup(header.bssid.clone());

                // TODO: Check why we get some empty SSIDs
                if access_point.ssid == "" {
                    return None; 
                }

                let node = Node::new(header.bssid.clone(), access_point.label.clone(), 0);
                access_point.nodes.push(node);
                self.net_map.insert(header.bssid, access_point.clone());
                return Some(access_point);
            }
        }
        return None;
    }

    fn add_to_collection(&mut self, mac: String, bssid: String, signal: i8) {
        if !mac.contains(BROADCAST) && !mac.starts_with(MULTICAST) {
            let node = self.add_node(mac.clone(), signal);
            let link = Link::new(mac, bssid.clone());
            if let Some(access_point) = self.net_map.get_mut(&bssid) {
                let mut node_iter = access_point.nodes.clone().into_iter();
                if node_iter.find(| ref mut x| x.mac == node.mac) == None {
                    access_point.push_node(node);
                }

                if !access_point.links.contains(&link) {
                    access_point.push_link(link);
                }
            }
        }
    }

    fn add_node(&mut self, mac: String, signal: i8) -> Node {
        let vendor = self.vendors.lookup(mac.clone());
        let node = Node::new(mac.clone(), vendor, signal);
        node
    }

    fn add_people(&mut self, src: String, freq: f32, signal: i8) {
        // Add phone vendors here but move for a Lazy Load initialization later
        let phone_vendors = vec![
            "Samsung Electronics Co.,Ltd", "Apple, Inc.", "HTC Corporation",
            "Huawei Symantec Technologies Co.,Ltd.", "Google, Inc.", "Microsoft",
            "Motorola (Wuhan) Mobility Technologies Communication Co., Ltd."
        ];

        // Get Mobile Phone vendor
        let vendor = self.vendors.lookup(src.clone());
        if phone_vendors.contains(&vendor.as_str()) {
            let distance = self.calc_distance(freq, signal);
            let person = People { 
                mac: src.clone(), vendor: vendor, signal: signal, distance: distance
            };
            self.people_map.insert(src, person);
        }
    }

    // https://en.wikipedia.org/wiki/Free-space_path_loss
    fn calc_distance(&mut self, freq: f32, signal: i8) -> f32 {
        let value = 10_f32;
        let expr = (FREE_SPACE_PATH_LOSS - (20.0*freq.log10()) + (-signal as f32)) / 20.0;
        value.powf(expr)
    }
}
