#![allow(unused_imports)]
#[allow(dead_code)]

mod network;
use crate::network::network_core::{analyse_interfaces, ping_host_syscmd, scan_ports_tcp};
use crate::network::network_helpers::{split_ip_range, create_ip_from_range};

use std::str::FromStr;
use std::time::Duration;
use std::net::{IpAddr, Ipv4Addr};
use ipnet::Ipv4Net;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};

use indicatif::{ProgressBar, ProgressStyle};
use tokio::task;
use futures::future::join_all;

use clap::{Parser, ArgAction};

#[derive(Parser)]
struct Cli {
    #[arg(help = "Either IPv4 address without subnet or CIDR notation. If ip_to not set, ip_from is scanned, otherwise range from ip_from to ip_to")]
    ip_from: Option<String>,
    
    #[arg(help = "If ip_from is a IPv4 address, this is the end of the range. Must be greater than ip_from")]
    ip_to: Option<String>,
    
    #[arg(help = "Timeout in milliseconds. Defaults to 100")]
    #[arg(short, long, default_value_t=100)]
    timeout: u32,
    
    #[arg(help = "If IP range is specified, number of IP addresses each worker receives. Defaults to 10")]
    #[arg(short, long, default_value_t=10)]
    chunksize: usize,
    
    #[arg(short, long, action = ArgAction::SetTrue)]
    verboose: bool,
}

fn print_results(results: &Vec<network::network_core::PortScanResult>, n_total: u32, n_up: u32) {
    
    println!("--------------------------------------------------------------------------------------------------------------------------------\n");
    println!("RESULTS:");
    println!("Total IPs scanned: {}", n_total);
    println!("IPs UP: {}", n_up);
    
    println!("--------------------------------------------------------------------------------------------------------------------------------\n");
    println!("IPs UP:");
    for result in results.iter() {
        if result.status == network::network_core::Status::Up {
            println!(
                "IP: {:?} ; Status: {:?} ; Hostname: {:?} ; Open TCP Ports: {:?}",
                result.ip_address,
                result.status,
                result.hostname.as_ref().unwrap(),
                result.open_ports.as_ref().unwrap(),
            );
        }
    };

    println!("\nIPs DOWN:");
    for result in results.iter() {
        if result.status == network::network_core::Status::Down {
            println!("IP: {:?} ; Status: {:?}", result.ip_address, result.status);
        }
    };
}


#[tokio::main]
async fn main() {

    let args = Cli::parse();

    let mut do_range: bool = false;

    let timeout = args.timeout;
    let chunksize = args.chunksize;

    // Always analyse network interfaces
    println!("--------------------------------------------------------------------------------------------------------------------------------\n");
    println!("Analyse interfaces ...");
    analyse_interfaces();
    println!("--------------------------------------------------------------------------------------------------------------------------------\n");

    // If neither IP from nor IP to are set, we're done
    if args.ip_from.is_none() && args.ip_to.is_none() {
        println!("No IP from or to specified, we're done");
        return
    }

    // Unwrap the ip_from into str
    let ip_from_string: String = args.ip_from.expect("IP from must be supplied");
    println!("{}", ip_from_string);

    let ip_from: Ipv4Addr;
    let ip_to: Option<Ipv4Addr>;

    // First unwrap the IP from address. It can be either a single IPv4 address or a pnet::Ipv4Net
    // Parse the input into either Ipv4Addr or Ipv4Net
    match ip_from_string.parse::<Ipv4Addr>() {
        Ok(ipv4_addr) => {
            // Handle the single IP case
            println!("Parsed as Ipv4Addr: {}", ipv4_addr);

            // Assign the value
            ip_from = ipv4_addr;
            println!("Assigned ip_from: {}", ip_from);

            // Check IP to
            match args.ip_to {
                Some(ip_to) => {

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
                        eprintln!("Invalid IP Range: {:?} to {:?}. Make sure ip_from is logically smaller than ip_to", ip_from, ip_to);
                        return;
                    }

                    println!("IP Range: {:?} to {:?} ; verboose {:?}", ip_from, ip_to, args.verboose);
                    do_range = true;
                    Some(ip_to)
                },
                None => {
                    println!("IP Single: {:?} ; verboose {:?}", ip_from, args.verboose);
                    None
                }
            };
        }
        Err(_) => match ip_from_string.parse::<Ipv4Net>() {
            Ok(ipv4_net) => {
                // Handle the network case
                println!("Parsed as Ipv4Net: {}", ipv4_net);

                // Extract first and last IP
                let ips: Vec<Ipv4Addr> = ipv4_net.hosts().collect();
                let ip_from: Ipv4Addr = ips[0];
                let ip_to: Ipv4Addr = ips[ips.len()-1];

                do_range = true;
            }
            Err(_) => {
                // Handle invalid input
                eprintln!(
                    "Failed to parse '{}' as either Ipv4Addr or Ipv4Net",
                    ip_from_string
                );
            }
        },
    }
    
    if !do_range {
        println!("Scanning single IP {:?}", ip_from);
        
        let success_ping = ping_host_syscmd(ip_from, timeout, true).await;
        println!("Status ping {:?} : {:?}", ip_from, success_ping);

        // // Test serializing and deserializing
        // let serialized = serde_json::to_string(&success_ping).unwrap();
        // println!("Serialized: {:?}", serialized);
        // let deserialized: network::network_core::PortScanResult = serde_json::from_str(&serialized).unwrap();
        // println!("Deserialized: {:?}", deserialized);

    } else {
        
        println!("Scanning IP Range {:?} to {:?}", &ip_from.to_string(),ip_to.to_string());

        // Split IP Range
        let (ip_ranges, n_ips) = split_ip_range(ip_from, ip_to, chunksize);

        let progress_bar = Arc::new(Mutex::new(ProgressBar::new(n_ips as u64)));
        progress_bar.lock().unwrap().set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({eta})")
                .expect("Invalid template format"),
        );

        // Run concurrently
        let shared_vector = Arc::new(Mutex::new(Vec::new()));
        let mut tasks = vec![];
        for range in ip_ranges {
            // Generate IPs for the current range
            let ip_to_check = create_ip_from_range(range);
            let vector = Arc::clone(&shared_vector);
            let pb = Arc::clone(&progress_bar);
            // Spawn a new async task for each IP range
            let task = task::spawn(async move {
                for ip_addr in ip_to_check {
                    let ip: Ipv4Addr = ip_addr.parse().unwrap();
                    // Call the async ping function
                    let ping_result = ping_host_syscmd(ip, timeout, false).await;
                    // Lock the vector to write the result
                    let mut locked_vector = vector.lock().unwrap();
                    locked_vector.push(ping_result);
                    // Lock pb and increment
                    let pb = pb.lock().unwrap();
                    pb.inc(1);
                }
            });

            tasks.push(task);
        }
        // Wait for all async tasks to finish
        join_all(tasks).await;
        // Print the results
        let mut locked_vector = shared_vector.lock().unwrap();
        
        // Pretty print the results
        // Sort by IP
        locked_vector.sort_by(|a, b| a.ip_address.cmp(&b.ip_address));

        // Gather information about how many IP scanned, how many are up etc
        let n_total: u32 = locked_vector.len() as u32;
        let mut n_up: u32 = 0;
        locked_vector.iter().for_each(|result| {
            if result.status == network::network_core::Status::Up {
                n_up += 1;
            }
        });

        // Print the results
        print_results(&locked_vector, n_total, n_up);
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