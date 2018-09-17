use errors::*;
use vendors::*;
use dot11::*;
use info::*;
use radiotap::Radiotap;
use std::collections::HashMap;

static PROTOCOL: &'static str = "802.11";
static BROADCAST: &'static str = "ff:ff:ff:ff:ff:ff";
static UNSPECIFIED: &'static str = "00:00:00:00:00:00";


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
    ssid: String,
    protocol: String,
    version: String,
    router_id: String, // BSSID
    vendor: String,
    signal: i32,
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
            vendor: String::new(),
            signal: 0, 
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
    pub vendor: String,
    pub signal: i32,
}

impl Node {
    fn new(mac: String, vendor: String, signal: i32) -> Node {
        Node {
            mac: mac,
            vendor: vendor,
            signal: signal
        }
    }
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

#[derive(Clone, Debug)]
pub struct Mapper {
    pub vendors: VendorsDB,
    pub net_map: HashMap<String, Collection>
}

impl Mapper {
    pub fn new() -> Result<Mapper> {
        let vendors = VendorsDB::from_file("data/oui.txt")?;

        Ok(Mapper {
            vendors: vendors,
            net_map: HashMap::new()
        })
    }

    pub fn map(&mut self, _radio_header: Radiotap, dot11_header: Dot11Header) {
        let info = dot11_header.info.clone();
        let frame_type = dot11_header.frame_control.frame_type;
        let frame_subtype = dot11_header.frame_control.frame_subtype;

        // AssoResp with status == 0 would provide all info we need for the Node/Link
        if frame_type == FrameType::Management && frame_subtype == FrameSubType::AssoResp {
            self.add_to_collection(dot11_header.dst, dot11_header.bssid);
        } else if frame_type == FrameType::Data {
            if frame_subtype == FrameSubType::QoS || frame_subtype == FrameSubType::Data {
                self.add_to_collection(dot11_header.src, dot11_header.bssid.clone());
                self.add_to_collection(dot11_header.dst, dot11_header.bssid);
            } else if frame_subtype == FrameSubType::NullData {
                // NullData informs Device Power Serving mode
                self.add_to_collection(dot11_header.dst, dot11_header.bssid);
            }           
        } else if frame_type == FrameType::Management {
            // Lets use the Beacon frame to get Access Point information
            if let BodyInformation::Beacon(beacon) = info.clone() {
                self.add_access_point(beacon.ssid.value, dot11_header);
            }
        }
    }

    fn add_access_point(&mut self, ssid: String, dot11_header: Dot11Header) {
        if !dot11_header.bssid.contains(BROADCAST) && !dot11_header.bssid.contains(UNSPECIFIED) {
            let header = dot11_header.clone();
            let mut access_point = Collection::new();

            access_point.ssid = ssid.clone();
            access_point.router_id = header.bssid.clone();
            access_point.vendor = self.vendors.lookup(header.bssid.clone());

            let node = Node::new(header.bssid.clone(), access_point.vendor.clone(), 0);
            access_point.nodes.push(node);
            self.net_map.insert(header.bssid, access_point);
        }
    }

    fn add_to_collection(&mut self, mac: String, bssid: String) {
        if !mac.contains(BROADCAST) {
            let node = self.add_node(mac.clone());
            let link = Link::new(mac, bssid.clone());
            if let Some(access_point) = self.net_map.get_mut(&bssid) {
                if !access_point.nodes.contains(&node) {
                    access_point.push_node(node);
                }

                if !access_point.links.contains(&link) {
                    access_point.push_link(link);
                }
            }
        }
    }

    fn add_node(&mut self, mac: String) -> Node {
        let vendor = self.vendors.lookup(mac.clone());
        let node = Node::new(mac.clone(), vendor, 0);
        node
    }
}

