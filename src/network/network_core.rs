use pnet::datalink::{self, NetworkInterface, Channel};
use pnet::packet::Packet;
use pnet::packet::ipv4::Ipv4Packet;
use std::collections::HashSet;

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

                let mut attempts = 0;
                let max_attempts = 3; // Set the maximum number of attempts

                while attempts < max_attempts {
                    if let Ok(packet) = rx.next() {
                        if let Some(ipv4_packet) = Ipv4Packet::new(packet) {
                            let source_ip = ipv4_packet.get_source();
                            active_hosts.insert(source_ip.to_string());
                            break; // Stop after receiving the first packet
                        }
                    } else {
                        // No packet received; increment the attempt counter
                        attempts += 1;
                    }
                }
            }
            _ => {
                eprintln!("Unsupported channel type");
            }
        }

        // match datalink::channel(&interface, Default::default()) {
        //     Ok(Channel::Ethernet(_, mut rx)) => {
        //         while let Ok(packet) = rx.next() {

        //             println!("Received packet: {:?}", packet);

        //             if let Some(ipv4_packet) = Ipv4Packet::new(packet) {
        //                 let source_ip = ipv4_packet.get_source();
        //                 active_hosts.insert(source_ip.to_string());
        //                 break;
        //             }
        //         }
        //     }
        //     Ok(_) => {
        //         eprintln!("Unsupported channel type");
        //     }
        //     Err(e) => {
        //         eprintln!("Failed to create datalink channel: {}", e);
        //     }
        // }    
    }

    active_hosts
}