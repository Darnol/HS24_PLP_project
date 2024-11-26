#![allow(unused_imports)]
#[allow(dead_code)]

use std::net::Ipv4Addr;

pub fn create_ip_from_range(range_ip: (String, String)) -> Vec<String> {
    let mut ip_list = vec![];
    
    let start: Ipv4Addr = range_ip.0.parse().unwrap();
    let end: Ipv4Addr = range_ip.1.parse().unwrap();
    let mut current_ip = start;
    
    while current_ip <= end {
        ip_list.push(current_ip.to_string());
        current_ip = Ipv4Addr::from(u32::from(current_ip) + 1);
    }
    
    ip_list
}

// TODO: Chunks > than total ips -> handle that
pub fn split_ip_range(start_ip: Ipv4Addr, end_ip: Ipv4Addr, chunks: usize) -> (Vec<(String, String)>, u32) {
    let mut ranges = vec![];
    let total_ips = (u32::from(end_ip) - u32::from(start_ip)) + 1;

    if chunks>total_ips as usize {
        panic!("Chunks cannot be greater than total IPs");
    }
    
    let chunk_size = (total_ips / chunks as u32) as usize;
    
    let mut current_ip = start_ip;
    
    while current_ip < end_ip {
        if u32::from(end_ip) - u32::from(current_ip) < chunk_size as u32 {
            ranges.push((current_ip.to_string(), end_ip.to_string()));
            break;
        }
        let next_ip = Ipv4Addr::from(u32::from(current_ip) + chunk_size as u32);
        ranges.push((current_ip.to_string(), next_ip.to_string()));
        current_ip = Ipv4Addr::from(u32::from(next_ip) + 1);
    }
    
    (ranges, total_ips)
}