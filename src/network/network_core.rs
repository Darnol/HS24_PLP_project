#![allow(unused_imports)]
#[allow(dead_code)]

use pnet::datalink::{self, NetworkInterface, Channel};
use pnet::packet::ipv4::Ipv4Packet;

use std::time::Duration;
use std::collections::HashSet;
use std::process::Command;
use std::net::{IpAddr, Ipv4Addr, TcpStream};

const TCP_PORTS: [u16; 10] = [20,21,22,23,25,53,80,110,143,443];

#[derive(Debug)]
enum Status {
    Up,
    Down,
}

#[derive(Debug)]
struct NetworkDevice {
    ip_address: Ipv4Addr,
    status: Status,
    open_ports: Option<Vec<u16>>, // Only populated if the device is "up"
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
    println!("Showing interfaces");

    // All interfaces
    let interfaces = datalink::interfaces();

    // Filter out loopback interfaces
    let interfaces_no_loopback: Vec<NetworkInterface> = interfaces.into_iter()
        .filter(|iface| !iface.is_loopback())
        .collect();

    for interface in interfaces_no_loopback {
        // Show the interface
        println!("Interface: {:?}", interface.description);

        // Print the IPs of possibly relevant interfaces
        for ipv4network in interface.ips {
            if ipv4network.ip() != Ipv4Addr::UNSPECIFIED {
                println!("-- Possible interesting IPv4 Address: {}/{}", ipv4network.ip(), ipv4network.prefix());
            }
        }
    }
}

pub async fn ping_host_syscmd(ip: IpAddr, timeout: u32, verboose: bool) -> bool {

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
        return true;
    } else {
        if verboose {
            println!("Ping failed");
        }
        return false;
    }
}

pub fn scan_ports_tcp(ip: IpAddr, timeout: Duration, ports: &[u16]) -> Vec<u16> {
    let mut open_ports: Vec<u16> = Vec::new();

    for port in ports {
        let address = format!("{}:{}", ip, port);
        match TcpStream::connect_timeout(
            &address.parse().unwrap(),
            timeout
        ) {
            Ok(_) => {
                println!("Port {} is open", port);
                open_ports.push(*port);
            }
            Err(_) => {
                println!("Port {} is closed", port);
            }
        }
    }

    open_ports
}

pub fn scan_interfaces() -> HashSet<String> {
    
    // All interfaces
    let interfaces = datalink::interfaces();

    // Filter out loopback interfaces
    let interfaces_no_loopback: Vec<NetworkInterface> = interfaces.into_iter()
        .filter(|iface| !iface.is_loopback())
        .filter(|iface| iface.name == "\\Device\\NPF_{2573BF24-C0DC-4565-A709-8D6EC53FC892}") // single out working interface
        .collect();

    let mut active_hosts = HashSet::new();

    for interface in interfaces_no_loopback {
        // Scan the single interface
        println!("Scanning interface: {}", interface.name);

        match datalink::channel(&interface, Default::default()).expect("Failed to create datalink channel") {
            Channel::Ethernet(_, mut rx) => {
                println!("Channel type: Ethernet");

                let mut packet_count = 0;
                const MAX_PACKETS: u32 = 10;

                while let Ok(packet) = rx.next() {
                    packet_count += 1;
                    println!("Received packet number: {:?}", packet_count);

                    if let Some(ipv4_packet) = Ipv4Packet::new(packet) {
                        println!("From {} to {}", ipv4_packet.get_source(), ipv4_packet.get_destination());
                    }

                    if packet_count >= MAX_PACKETS {
                        break;
                    }
                }
            }
            _ => {
                eprintln!("Unsupported channel type");
            }
        }
    }

    active_hosts
}