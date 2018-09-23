use errors::*;
use std::fs::File;
use std::io::{BufReader, prelude::*};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct VendorsDB {
    db: HashMap<String, String>
}

impl VendorsDB {
    pub fn from_file(file_name: &str) -> Result<VendorsDB> {
        let mut vendors: HashMap<String, String> = HashMap::new();
        let file = File::open(file_name)?;
        let buf_reader = BufReader::new(file);

        for line in buf_reader.lines() {
            let strline = line?;
            let v: Vec<&str> = strline.split('\t').collect();
            if v[0].contains("base 16") {
                let (vendor_code, _discard) = v[0].split_at(6);
                let vendor_name = v[2];
                vendors.insert(vendor_code.to_lowercase(), vendor_name.to_string());
            }
        }

        Ok(VendorsDB {
            db: vendors
        })
    }

    pub fn lookup(&self, mac: String) -> String {
        let v: Vec<&str> = mac.split(':').collect();
        let key: String = format!("{}{}{}", v[0], v[1], v[2]);

        match self.db.get(&key) {
            Some(key) => key.clone(),
            None => "".into()
        }
    }
}