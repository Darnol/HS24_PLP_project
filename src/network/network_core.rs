#![allow(unused_imports)]
#[allow(dead_code)]

use pnet::datalink::{self, NetworkInterface, Channel};
use pnet::packet::ipv4::Ipv4Packet;

use std::time::Duration;
use std::collections::HashSet;
use std::process::Command;
use std::net::{IpAddr, Ipv4Addr, TcpStream};
use dns_lookup::lookup_addr;

use serde::{Serialize, Deserialize};

const TCP_PORTS: [u16; 11] = [20,21,22,23,25,53,80,110,143,443,445];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Status {
    Up,
    Down,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortScanResult {
    pub ip_address: Ipv4Addr,
    pub status: Status,
    pub hostname: Option<String>, // Only populated if the device is "up"
    pub open_ports: Option<Vec<u16>>, // Only populated if the device is "up"
}

impl PortScanResult {
    fn new(ip_address: Ipv4Addr, status: Status, hostname: Option<String>, open_ports: Option<Vec<u16>>) -> Self {
        PortScanResult {
            ip_address,
            status: status.clone(),
            hostname: if let Status::Up = status { hostname } else { None },
            open_ports: if let Status::Up = status { open_ports } else { None },
        }
    }
}

fn create_ping_command(ip_str: &String, timeout: u32) -> String {
    if cfg!(target_os = "windows") {
        format!(
            "ping {} -n 4 -w {} >nul && exit 0 || exit 1",
            ip_str,
            timeout
        )
    } else if cfg!(target_os = "macos") {
        format!(
            "ping -c 2 {} >/dev/null && exit 0 || exit 1",
            ip_str
        )
    } else {
        panic!("Unsupported OS detected");
    }
}

fn determine_ping_parameters() -> (String, String) {
    if cfg!(target_os = "windows") {
        (String::from("cmd"), String::from("/C"))
    } else if cfg!(target_os = "macos") {
        (String::from("sh"), String::from("-c"))
    } else {
        panic!("Unsupported OS detected");
    }
}

pub fn analyse_interfaces() -> () {
    // All interfaces
    let interfaces = datalink::interfaces();

    // Filter out loopback interfaces and interfaces that are not up
    let interfaces_no_loopback: Vec<NetworkInterface> = interfaces.into_iter()
        .filter(|iface| !iface.is_loopback())
        .filter(|iface| iface.is_up())
        .filter(|iface| iface.ips.len() > 0)
        .collect();

    for interface in interfaces_no_loopback {

        // Show the interface description
        let interface_text = if !interface.description.is_empty() {
            format!("Interface: {} - {}", interface.name, interface.description)
        } else {
            format!("Interface: {}", interface.name)
        };

        println!("{}", interface_text);

        // Print the IPs of possibly relevant interfaces
        for ipv4network in interface.ips {
            if ipv4network.ip().is_ipv4() {
                println!("-- Possible interesting IPv4 Address: {}/{}", ipv4network.ip(), ipv4network.prefix());
            }
        }
    }
}

pub async fn ping_host_syscmd(ip: Ipv4Addr, timeout: u32, verboose: bool) -> PortScanResult {

    // ip to String
    let ip_str = ip.to_string();

    if verboose {
        println!("Pinging host: {:?}", ip);
        println!("Timeout: {:?}", timeout);
    }

    // Determine OS and fetch command for ping
    let command: String = create_ping_command(&ip_str, timeout);
    
    if verboose {
        println!("Command: {}", command);
    }

    let (shell, flag) = determine_ping_parameters();

    let status = Command::new(shell)
        .args([flag, command])
        .status()
        .expect("Failed to execute command");

    if status.success() {
        if verboose {
            println!("Ping successful");
        }

        // Get hostname
        let hostname = match lookup_addr(&IpAddr::from(ip)) {
            Ok(name) => name,
            Err(_) => String::from("Unknown"),
        };

        // Scan common TCP ports
        let open_ports: Vec<u16> = scan_ports_tcp(ip, Duration::from_millis(100), &TCP_PORTS);
        return PortScanResult::new(ip.to_string().parse().unwrap(), Status::Up, Some(hostname), Some(open_ports));
    } else {
        if verboose {
            println!("Ping failed");
        }
        return PortScanResult::new(ip.to_string().parse().unwrap(), Status::Down, None, None);
    }
}

pub fn scan_ports_tcp(ip: Ipv4Addr, timeout: Duration, ports: &[u16]) -> Vec<u16> {
    let mut open_ports: Vec<u16> = Vec::new();

    for port in ports {
        let address = format!("{}:{}", ip, port);
        match TcpStream::connect_timeout(
            &address.parse().unwrap(),
            timeout
        ) {
            Ok(_) => {
                open_ports.push(*port);
            }
            Err(_) => {}
        }
    }

    open_ports
}