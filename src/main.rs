mod network;
use crate::network::network_core::{analyse_interfaces, ping_host_syscmd, split_ip_range, create_ip_from_range};

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {

    analyse_interfaces();

    // // DOES NOT WORK
    // let active_hosts = scan_interfaces();
    // println!("Active hosts: {:?}", active_hosts);

    // // Test ping
    // let ip: IpAddr = "1.1.1.1".parse().unwrap();
    // // let ip: IpAddr = "1.2.3.4".parse().unwrap();
    // println!("Pinging host: {:?}", ip);
    // let timeout: u32 = 100;
    // println!("Timeout: {:?}", timeout);
    // let success_ping = ping_host_syscmd(ip, timeout);
    // println!("Status ping {:?} : {:?}", ip, success_ping);

    // // Test IP Range splitting
    // let ip_start = "192.168.0.1";
    // let ip_end = "192.168.0.254";
    // let ip_ranges = split_ip_range(ip_start, ip_end, 10);
    // println!("IP Ranges: {:?}", ip_ranges);

    // // Test IP Range creation
    // let ip_list = create_ip_from_range(ip_ranges.clone().into_iter().nth(0).unwrap());
    // println!("IP List: {:?}", ip_list);




    // // Test concurrency
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
    //             let success_ping = ping_host_syscmd(ip, timeout);
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

}