use pnet::datalink::{self, NetworkInterface, Channel};
use pnet::packet::ipv4::Ipv4Packet;

use std::collections::HashSet;
use std::process::Command;
use std::net::IpAddr;

pub fn show_interfaces() -> () {
    println!("Showing interfaces");

    // All interfaces
    let interfaces = datalink::interfaces();

    // Filter out loopback interfaces
    let interfaces_no_loopback: Vec<NetworkInterface> = interfaces.into_iter()
        .filter(|iface| !iface.is_loopback())
        .collect();

    for interface in interfaces_no_loopback {
        // Show the interface
        println!("Interface: {:?}", interface);
    }
}

pub fn ping_host_syscmd(ip: IpAddr, timeout: u32) -> u32 {

    // ip to String
    let ip_str = ip.to_string();

    println!("ping ip {}", ip_str);

    // Determine OS
    if cfg!(target_os = "windows") {
        println!("Windows OS detected");

        let command = format!(
            "ping {} -n 4 -w {} >nul && exit 0 || exit 1",
            ip_str,
            timeout
        );

        println!("Command: {}", command);

        let status = Command::new("cmd")
            .args(["/C", &command])
            .status()
            .expect("Failed to execute command");

        if status.success() {
            println!("Ping successful");
            return 0;
        } else {
            println!("Ping failed");
            return 1;
        }

    } else if cfg!(target_os = "macos") {
        println!("Mac OS detected");

        let command = format!(
            "ping -c 2 {} >/dev/null && exit 0 || exit 1",
            ip_str
        );

        println!("Command: {}", command);

        let status = Command::new("sh")
            .args(["-c", &command])
            .status()
            .expect("Failed to execute command");

        if status.success() {
            println!("Ping successful");
            return 0;
        } else {
            println!("Ping failed");
            return 1;
        }        

    } else {
        panic!("Unsupported OS detected");
    }
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
                const max_packets: u32 = 10;

                while let Ok(packet) = rx.next() {
                    packet_count += 1;
                    println!("Received packet number: {:?}", packet_count);

                    if let Some(ipv4_packet) = Ipv4Packet::new(packet) {
                        println!("From {} to {}", ipv4_packet.get_source(), ipv4_packet.get_destination());
                    }

                    if packet_count >= max_packets {
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

pub fn split_ip_range(start_ip: &str, end_ip: &str, chunks: usize) -> Vec<(String, String)> {
    let start: std::net::Ipv4Addr = start_ip.parse().unwrap();
    let end: std::net::Ipv4Addr = end_ip.parse().unwrap();
    
    let mut ranges = vec![];
    let total_ips = (u32::from(end) - u32::from(start)) + 1;
    
    if chunks>total_ips as usize {
        panic!("Chunks cannot be greater than total IPs");
    }
    
    let chunk_size = (total_ips / chunks as u32) as usize;

    let mut current_ip = start;

    while current_ip < end {
        if u32::from(end) - u32::from(current_ip) < chunk_size as u32 {
            ranges.push((current_ip.to_string(), end.to_string()));
            break;
        }
        let next_ip = std::net::Ipv4Addr::from(u32::from(current_ip) + chunk_size as u32);
        ranges.push((current_ip.to_string(), next_ip.to_string()));
        current_ip = std::net::Ipv4Addr::from(u32::from(next_ip) + 1);
    }

    ranges
}

pub fn create_ip_from_range(range_ip: (String, String)) -> Vec<String> {
    let mut ip_list = vec![];

    let start: std::net::Ipv4Addr = range_ip.0.parse().unwrap();
    let end: std::net::Ipv4Addr = range_ip.1.parse().unwrap();
    let mut current_ip = start;
    
    while current_ip <= end {
        ip_list.push(current_ip.to_string());
        current_ip = std::net::Ipv4Addr::from(u32::from(current_ip) + 1);
    }

    ip_list
}