mod network;
use crate::network::network_core::{show_interfaces, scan_interfaces, ping_host, ping_host_syscmd};

use std::process::Command;
use std::time::Duration;
use std::net::IpAddr;
use ping::ping;

fn main() {
    // show_interfaces();

    // let active_hosts = scan_interfaces();
    // println!("Active hosts: {:?}", active_hosts);

    // Test ping
    let ip: IpAddr = "192.168.0.13".parse().unwrap();
    println!("Pinging host: {:?}", ip);

    let timeout = Duration::from_secs(1);
    println!("Timeout: {:?}", timeout);

    ping_host_syscmd(ip);

    // match ping(ip, Some(timeout), Some(166), Some(3), Some(5), None) {
    //     Ok(duration) => println!("Ping response time: {:?}", duration),
    //     Err(e) => println!("Ping failed: {}", e),
    // }

    // ping_host(ip, timeout);

}