mod network;
use crate::network::network_core::{show_interfaces, scan_interfaces};

fn main() {
    show_interfaces();
    let active_hosts = scan_interfaces();
    println!("Active hosts: {:?}", active_hosts);
}