#![allow(unused_imports)]
#[allow(dead_code)]

mod network;
use crate::network::network_core::{analyse_interfaces, ping_host_syscmd, scan_ports_tcp};
use crate::network::network_helpers::{split_ip_range, create_ip_from_range};

use std::time::Duration;
use std::collections::HashMap;
use std::net::{IpAddr};
use std::sync::{Arc, Mutex};
use std::thread;

use tokio::task;
use futures::future::join_all;

const TCP_PORTS: [u16; 10] = [20,21,22,23,25,53,80,110,143,443];

#[tokio::main]
async fn main() {

    // analyse_interfaces();

    // // DOES NOT WORK
    // let active_hosts = scan_interfaces();
    // println!("Active hosts: {:?}", active_hosts);

    // Test ping
    let ip: IpAddr = "1.1.1.1".parse().unwrap();
    // let ip: IpAddr = "1.2.3.4".parse().unwrap();
    println!("Pinging host: {:?}", ip);
    let timeout: u32 = 100;
    println!("Timeout: {:?}", timeout);
    let success_ping = ping_host_syscmd(ip, timeout, true).await;
    println!("Status ping {:?} : {:?}", ip, success_ping);

    // // Test IP Range splitting
    // let ip_start = "192.168.0.1";
    // let ip_end = "192.168.0.254";
    // let ip_ranges = split_ip_range(ip_start, ip_end, 10);
    // for range in ip_ranges.clone() {
    //     println!("IP Range: {:?}", range);
    // }

    // // Test IP Range creation on the first range
    // let ip_list = create_ip_from_range(ip_ranges.clone().into_iter().nth(0).unwrap());
    // for ip in ip_list.clone() {
    //     println!("IP: {:?}", ip);
    // }



    // // Test TCP Port scanning
    // // let ip: IpAddr = "35.180.139.74".parse().unwrap();
    // let ip: IpAddr = "1.1.1.1".parse().unwrap();
    // let timeout: Duration = Duration::from_millis(1000);
    // let open_ports: Vec<u16> = scan_ports_tcp(ip, timeout, &TCP_PORTS);
    // println!("Open ports: {:?}", open_ports);





    // // Test running sequentially
    // let ips_to_ping = create_ip_from_range( (String::from("192.168.0.1"), String::from("192.168.0.254")) );
    // let mut results_concurrent = HashMap::new();
    // for ip_addr in ips_to_ping {
    //     let ip: IpAddr = ip_addr.parse().unwrap();
    //     let timeout: u32 = 100;
    //     let success_ping = ping_host_syscmd(ip, timeout, false);
    //     println!("Status ping {:?} : {:?}", ip, success_ping);
    //     results_concurrent.insert(ip.to_string(), success_ping);
    // }




    // // Test concurrency with threads
    // let shared_map = Arc::new(Mutex::new(HashMap::new()));
    // let mut handles = vec![]; // This holds the handles, which are separate threads
    // for range in ip_ranges {

    //     let ip_to_check = create_ip_from_range(range);
        
    //     let map = Arc::clone(&shared_map);

    //     // Spawn a new thread for each IP range
    //     let handle = thread::spawn(move || {

    //         // Each threads will loop through IP addresses
    //         for ip_addr in ip_to_check {
    //             let ip: IpAddr = ip_addr.parse().unwrap();
    //             let timeout: u32 = 100;
    //             let success_ping = ping_host_syscmd(ip, timeout, false);
    //             println!("Status ping {:?} : {:?}", ip, success_ping);

    //             // Lock the map to write the result
    //             let mut locked_map = map.lock().unwrap();
    //             locked_map.insert(ip.to_string(), success_ping);
    //         }
            
    //     });

    //     handles.push(handle);
    // }

    // // Wait for all threads to finish
    // for handle in handles {
    //     handle.join().unwrap();
    // }

    // // Print the results
    // let locked_map = shared_map.lock().unwrap();
    // println!("Results: {:?}", *locked_map);




    // // Test concurrency with tokio
    // let shared_map = Arc::new(Mutex::new(HashMap::new()));
    // let mut tasks = vec![];
    // for range in ip_ranges {
    //     // Generate IPs for the current range
    //     let ip_to_check = create_ip_from_range(range);
    //     let map = Arc::clone(&shared_map);

    //     // Spawn a new async task for each IP range
    //     let task = task::spawn(async move {
    //         for ip_addr in ip_to_check {
    //             let ip: IpAddr = ip_addr.parse().unwrap();
    //             let timeout: u32 = 100;

    //             // Call the async ping function
    //             let success_ping = ping_host_syscmd(ip, timeout, false).await;
    //             println!("Status ping {:?} : {:?}", ip, success_ping);

    //             // Lock the map to write the result
    //             let mut locked_map = map.lock().unwrap();
    //             locked_map.insert(ip.to_string(), success_ping);
    //         }
    //     });

    //     tasks.push(task);
    // }

    // // Wait for all async tasks to finish
    // join_all(tasks).await;

    // // Print the results
    // let locked_map = shared_map.lock().unwrap();
    // println!("Results: {:?}", *locked_map);

}