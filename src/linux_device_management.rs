use errors::*;
use std::io::prelude::*;
use std::fs::{self, File};
use std::process::Command;
use std::path::{Path, PathBuf};

const ADAPTER_MONITOR_MODE: i32 = 803; // ARPHRD_IEEE80211_RADIOTAP
static NETWORK_INTERFACE_PATH: &'static str = "/sys/class/net";

#[derive(Clone, Debug)]
pub struct NetworkInterface {
    name: String,
    path: PathBuf,
    wireless: bool,
}

impl NetworkInterface {
    pub fn new<S>(network_interface: S) -> Result<(NetworkInterface)>
        where S: Into<String>    
    { 
        let network_interface_name = network_interface.into();
        let network_interface_path = Path::new(NETWORK_INTERFACE_PATH).join(network_interface_name.clone());
        if !network_interface_path.exists() {
            bail!("Network Interface not found")
        }

        let wireless = network_interface_path.join("wireless").exists();

        Ok(NetworkInterface {
            name: network_interface_name,
            path: network_interface_path,
            wireless: wireless,
        })
    }

    pub fn monitor_mode_on(&self) -> Result<()> {
        self.set_interface_mode("monitor")?;
        Ok(())
    }

    pub fn monitor_mode_off(&self) -> Result<()> {
        self.set_interface_mode("managed")?;
        Ok(())
    }
    
    fn set_interface_mode(&self, mode: &str) -> Result<()> {
        let _down_status = Command::new("ifconfig").arg(self.name.clone()).arg("down").status()?;

        let _iwconfig = Command::new("iwconfig").arg(self.name.clone()).arg("mode").arg(mode).status()?;

        let _up_status = Command::new("ifconfig").arg(self.name.clone()).arg("up").status()?;

        Ok(())
    }

    fn is_monitor_mode_device(&self, entry: String) -> Result<(bool)> {
        let mut str_mode = String::new();
        let path = format!("{}/{}/type", NETWORK_INTERFACE_PATH, entry);
        let mut file = File::open(&path)?;
        
        file.read_to_string(&mut str_mode)?;
        let mode: Vec<&str> = str_mode.split('\n').collect();
        let mode_number: i32 = mode[0].parse::<i32>().unwrap();

        Ok(mode_number == ADAPTER_MONITOR_MODE)
    }

    pub fn find_monitor_interfaces(&self) -> Result<()> {
        for entry in fs::read_dir(NETWORK_INTERFACE_PATH)? 
        {
            let filename = entry?.file_name().into_string().unwrap();
            if let Ok(found) = self.is_monitor_mode_device(filename.clone()) {
                if found {
                    println!("{:?}", filename);
                }
            }
        }

        Ok(())
    }
}