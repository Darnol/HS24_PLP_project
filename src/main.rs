#![allow(unused_imports)]
#[allow(dead_code)]

mod network;
use crate::network::network_core::{analyse_interfaces, ping_host_syscmd, scan_ports_tcp};
use crate::network::network_helpers::{split_ip_range, create_ip_from_range};

use std::str::FromStr;
use std::time::Duration;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Arc, Mutex};

use tokio::task;
use futures::future::join_all;

use clap::{Parser, ArgAction};

#[derive(Parser)]
struct Cli {
    /// The pattern to look for
    ip_from: String,
    ip_to: Option<String>,
    #[arg(short, long)]
    timeout: Option<u32>,
    #[arg(short, long, action = ArgAction::SetTrue)]
    verboose: bool,
}


#[tokio::main]
async fn main() {

    let args = Cli::parse();

    let mut do_range: bool = false;

    // Set timeout
    let timeout: u32 = match args.timeout {
        Some(timeout) => timeout,
        None => 100,
    };

    // First check IP from
    let ip_from: Ipv4Addr = match args.ip_from.parse() {
        Ok(ip) => ip,
        Err(_) => {
            eprintln!("Error: {} is not a valid IPv4 address.", args.ip_from);
            return;
        }
    };

    // Check IP to
    let ip_to: Option<Ipv4Addr> = match args.ip_to {
        Some(ref ip_to) => {

            // Check if the IP to is valid
            let _: Ipv4Addr = match ip_to.parse() {
                Ok(ip) => ip,
                Err(_) => {
                    eprintln!("Error: {} is not a valid IPv4 address.", ip_to);
                    return;
                }
            };

            // Check if the range is valid
            let ip_to: Ipv4Addr = ip_to.parse().unwrap();
            if ip_from >= ip_to {
                println!("Invalid IP Range: {:?} to {:?}. Make sure ip_from is logically smaller than ip_to", ip_from, ip_to);
                return;
            }

            println!("IP Range: {:?} to {:?} ; verboose {:?}", args.ip_from, ip_to, args.verboose);
            do_range = true;
            Some(ip_to)
        },
        None => {
            println!("IP Single: {:?} ; verboose {:?}", args.ip_from, args.verboose);
            None
        }
    };
    println!("--------------------------------------------------------------------------------------------------------------------------------");
    
    println!("Analyse interfaces ...");
    analyse_interfaces();
    println!("--------------------------------------------------------------------------------------------------------------------------------");


    if !do_range {
        println!("Scanning single IP {:?}", ip_from);

        let success_ping = ping_host_syscmd(ip_from, timeout, true).await;
        println!("Status ping {:?} : {:?}", ip_from, success_ping);
    } else {
        // TODO: Handle IP Range
        // Split Range
        // Execute concurrently
        // Gather results and print them nicely
    }


    
    // // Test ping
    // // let ip: IpAddr = "1.1.1.1".parse().unwrap();
    // // let ip: IpAddr = "8.8.8.8".parse().unwrap();
    // let ip: IpAddr = "198.252.206.16".parse().unwrap(); // Stackoverflow
    // // let ip: IpAddr = "1.2.3.4".parse().unwrap();
    // println!("Pinging host: {:?}", ip);
    // let timeout: u32 = 100;
    // println!("Timeout: {:?}", timeout);
    // let success_ping = ping_host_syscmd(ip, timeout, true).await;
    // println!("Status ping {:?} : {:?}", ip, success_ping);

    // // Test IP Range splitting
    // let ip_start = "192.168.0.1";
    // let ip_end = "192.168.0.254";
    // let ip_ranges = split_ip_range(ip_start, ip_end, 10);
    // // for range in ip_ranges.clone() {
    // //     println!("IP Range: {:?}", range);
    // // }

    // // Test IP Range creation on the first range
    // let ip_list = create_ip_from_range(ip_ranges.clone().into_iter().nth(0).unwrap());
    // // for ip in ip_list.clone() {
    // //     println!("IP: {:?}", ip);
    // // }

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


    // // Test concurrency with tokio
    // let shared_vector = Arc::new(Mutex::new(Vec::new()));
    // let mut tasks = vec![];
    // for range in ip_ranges {
    //     // Generate IPs for the current range
    //     let ip_to_check = create_ip_from_range(range);
    //     let vector = Arc::clone(&shared_vector);

    //     // Spawn a new async task for each IP range
    //     let task = task::spawn(async move {
    //         for ip_addr in ip_to_check {
    //             let ip: IpAddr = ip_addr.parse().unwrap();
    //             let timeout: u32 = 100;

    //             // Call the async ping function
    //             let ping_result = ping_host_syscmd(ip, timeout, false).await;
    //             println!("Status ping {:?} : {:?}", ip, ping_result.status);

    //             // Lock the vector to write the result
    //             let mut locked_vector = vector.lock().unwrap();
    //             locked_vector.push(ping_result);
    //         }
    //     });

    //     tasks.push(task);
    // }

    // // Wait for all async tasks to finish
    // join_all(tasks).await;

    // // Print the results
    // let locked_vector = shared_vector.lock().unwrap();
    // println!("Results: {:?}", *locked_vector);

}