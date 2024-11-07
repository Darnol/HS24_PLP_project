use pnet::datalink::{self, NetworkInterface, Channel};
use pnet::packet::ipv4::Ipv4Packet;

use std::collections::HashSet;
use std::process::Command;
use std::time::Duration;
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

pub fn ping_host_syscmd(ip: IpAddr, timeout: u32) -> () {

    // ip to String
    let ip_str = ip.to_string();

    println!("ping ip {}", ip_str);

    // Determine OS
    if cfg!(target_os = "windows") {
        println!("Windows OS detected");

        let command = format!(
            "ping {} -n 1 -w {} >nul && echo Ping succeeded || echo Ping failed",
            ip_str,
            timeout
        );

        println!("Command: {}", command);

        let output = Command::new("cmd")
            .args(["/C", &command])
            .output()
            .expect("Failed to execute command");

        let result = String::from_utf8_lossy(&output.stdout);
        println!("{}", result);

    } else if cfg!(target_os = "macos") {
        println!("Mac OS detected");
    } else if cfg!(target_os = "linux") {
        panic!("Unsupported OS detected");
    }

    // let output = Command::new("ping")
    //     .arg(ip_str)
    //     .output()
    //     .expect("Failed to execute ping");

    // if output.status.success() {
    //     println!("Ping output:\n{}", String::from_utf8_lossy(&output.stdout));
    // } else {
    //     println!("Ping failed:\n{}", String::from_utf8_lossy(&output.stderr));
    // }
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