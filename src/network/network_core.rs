use pnet::datalink::{self, NetworkInterface, Channel};
use pnet::packet::ipv4::Ipv4Packet;

use std::collections::HashSet;
use std::process::Command;
use std::time::Duration;
use std::net::IpAddr;

use ping::ping;
use rand::random;

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

pub fn ping_host(ip: IpAddr, timeout: Duration) -> () {
    let ping_stream = ping(ip, Some(timeout), Some(166), Some(3), Some(5), Some(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23])).unwrap();
    println!("Ping stream: {:?}", ping_stream);

    // let ping_stream = ping(ip, None).expect("Failed to create ping");
    // for message in ping_stream {
    //     match message {
    //         pinger::PingResult::Pong(duration, _) => {
    //             println!("Duration: {:?}", duration)
    //         }
    //         _ => {} // Handle errors, log ping timeouts, etc.
    //     } 
    // }
}

pub fn ping_host_syscmd(ip: IpAddr) -> () {
    let output = Command::new("ping")
        .arg("8.8.8.8")
        .output()
        .expect("Failed to execute ping");

    if output.status.success() {
        println!("Ping output:\n{}", String::from_utf8_lossy(&output.stdout));
    } else {
        println!("Ping failed:\n{}", String::from_utf8_lossy(&output.stderr));
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