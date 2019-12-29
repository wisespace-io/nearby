use errors::*;
use std::thread;
use std::time::Duration;
use std::io::prelude::*;
use std::fs::{self, File};
use std::process::Command;
use std::path::{Path, PathBuf};

const ADAPTER_MONITOR_MODE: i32 = 803; // ARPHRD_IEEE80211_RADIOTAP
static NETWORK_INTERFACE_PATH: &'static str = "/sys/class/net";

#[derive(Clone, Debug)]
pub struct NetworkInterface {
    pub name: String,
    pub path: PathBuf,
    pub channels: Vec<String>,
    wireless: bool,
}

impl NetworkInterface {
    pub fn new<S>(network_interface: S) -> Result<NetworkInterface>
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
            channels: Vec::new(),
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

    fn is_monitor_mode_device(&self, entry: String) -> Result<bool> {
        let mut str_mode = String::new();
        let path = format!("{}/{}/type", NETWORK_INTERFACE_PATH, entry);
        let mut file = File::open(&path)?;

        file.read_to_string(&mut str_mode)?;
        let mode: Vec<&str> = str_mode.split('\n').collect();
        let mode_number: i32 = mode[0].parse::<i32>()?;

        Ok(mode_number == ADAPTER_MONITOR_MODE)
    }

    pub fn find_monitor_interfaces(&self) -> Result<()> {
        for entry in fs::read_dir(NETWORK_INTERFACE_PATH)? {
            let filename = entry?.file_name().into_string().unwrap();
            if let Ok(found) = self.is_monitor_mode_device(filename.clone()) {
                if found {
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn find_supported_channels(&mut self) -> Result<()> {
        let iwlist = Command::new("iwlist").arg(self.name.clone()).arg("freq").output()?;
        let output = String::from_utf8_lossy(&iwlist.stdout);
        let lines: Vec<&str> = output.split('\n').collect();
      
        for line in lines {
            let channels: Vec<String> = line.split(" : ").map(|s| s.into()).collect();
            if channels[0].contains(" Channel ") {
                let ch = channels[0].trim().replace("Channel ", "");
                self.channels.push(ch);
            }
        }
        Ok(())
    }

    pub fn start_channel_switch(&self) {
        let name = self.name.clone();
        let channels = self.channels.clone();
        let mut index = 0;

        let _handle = thread::spawn(move || {
            loop {
                index = (index + 1) % channels.len();
                let _cmd_status = Command::new("iwconfig").arg(name.clone())
                                                          .arg("channel")
                                                          .arg(channels[index].clone())
                                                          .status().unwrap();
                thread::sleep(Duration::from_millis(4000)); // 4 seconds per channel
            }
        });
    }
}