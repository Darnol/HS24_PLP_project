use pnet::datalink::{self, NetworkInterface, Channel};
use pnet::packet::ipv4::Ipv4Packet;

use std::collections::HashSet;
use std::process::Command;
use std::net::{IpAddr, Ipv4Addr};

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

pub fn ping_host_syscmd(ip: IpAddr, timeout: u32, verboose: bool) -> bool {

    // ip to String
    let ip_str = ip.to_string();

    if verboose {
        println!("Pinging host: {:?}", ip);
        println!("Timeout: {:?}", timeout);
    }

    // Determine OS
    if cfg!(target_os = "windows") { 
        
        let command = format!(
            "ping {} -n 4 -w {} >nul && exit 0 || exit 1",
            ip_str,
            timeout
        );
       
        if verboose {
            println!("Windows OS detected");
            println!("Command: {}", command);
        }

        let status = Command::new("cmd")
            .args(["/C", &command])
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

    } else if cfg!(target_os = "macos") {
        
        let command = format!(
            "ping -c 2 {} >/dev/null && exit 0 || exit 1",
            ip_str
        );
        
        if verboose {
            println!("Mac OS detected");
            println!("Command: {}", command);
        }

        let status = Command::new("sh")
            .args(["-c", &command])
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