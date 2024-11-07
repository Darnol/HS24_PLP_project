mod network;
use crate::network::network_core::{show_interfaces, scan_interfaces, ping_host_syscmd};

use std::net::IpAddr;

fn main() {
    // show_interfaces();

    // let active_hosts = scan_interfaces();
    // println!("Active hosts: {:?}", active_hosts);

    // Test ping
    let ip: IpAddr = "1.1.1.1".parse().unwrap();
    // let ip: IpAddr = "1.2.3.4".parse().unwrap();
    println!("Pinging host: {:?}", ip);

    let timeout: u32 = 100;
    println!("Timeout: {:?}", timeout);

    let success_ping = ping_host_syscmd(ip, timeout);
    println!("Status ping {:?} : {:?}", ip, success_ping);
}