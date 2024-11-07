use pnet::datalink::{self, NetworkInterface};
use pnet::packet::Packet;
use pnet::packet::ipv4::Ipv4Packet;

pub fn show_interfaces() -> () {
    println!("Showing interfaces");
    let interfaces = datalink::interfaces();
    let interface = interfaces.into_iter()
        .filter(|iface| !iface.is_loopback()) // Skip loopback interface
        .next()
        .expect("No network interface found");
    println!("Interface: {:?}", interface);
}