#![allow(dead_code)]

mod network;
use crate::network::network_core::{analyse_interfaces, ping_host_surge};
use crate::network::network_helpers::{split_ip_range, create_ip_from_range, create_ipv4_range};

use std::net::{Ipv4Addr, IpAddr};
use ipnet::Ipv4Net;
use std::sync::{Arc, Mutex};
use surge_ping::{Client, Config};

use indicatif::{ProgressBar, ProgressStyle};
use tokio::task;
use futures::{future::join_all, stream, StreamExt};

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

fn parse_ip_input(ip_from: &str, ip_to: Option<String>) -> (bool, Ipv4Addr, Option<Ipv4Addr>) {
    // Try ip_from as Ipv4Net
    if let Ok(ipv4_net) = ip_from.parse::<Ipv4Net>() {
        let ips: Vec<Ipv4Addr> = ipv4_net.hosts().collect();
        let ip_from: Ipv4Addr = ips[0];
        let ip_to: Ipv4Addr = ips[ips.len()-1];
        println!("Parsed ip_from as CIDR: {} to {}", ip_from, ip_to);
        return (true, ip_from, Some(ip_to));
    }

    // If not Ipv4Net, try Ipv4Addr
    match ip_from.parse::<Ipv4Addr>() {
        Ok(ipv4_addr) => {
            let ip_from: Ipv4Addr = ipv4_addr;
            println!("Parsed ip_from as Ipv4Addr: {}", ip_from);

            // Now check if ip_to is set and valid
            if let Some(ip_to) = ip_to {
                match ip_to.parse::<Ipv4Addr>() {
                    Ok(ipv4_addr) => {
                        println!("Parsed ip_to as Ipv4Addr: {}", ipv4_addr);
                        let ip_to: Ipv4Addr = ipv4_addr;
                        if ip_from >= ip_to {
                            panic!("Invalid IP Range: {:?} to {:?}. Make sure ip_from is logically smaller than ip_to", ip_from, ip_to);
                        }
                        return (true, ip_from, Some(ip_to));
                    },
                    Err(_) => {
                        panic!("Failed to parse ip_to '{}' as Ipv4Addr", ip_to);
                    }
                }
            } else {
                return (false, ip_from, None);
            }
            
        }
        Err(_) => {
            // Handle invalid input ip_form
            panic!("Failed to parse ip_from '{}' as either Ipv4Addr or Ipv4Net",ip_from);
        }
    }
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
                result.hostname,
                result.open_tcp_ports,
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

    // Unwrap the ip_from into str and parse the inputs
    let ip_from_string: String = args.ip_from.expect("IP from must be supplied");
    let (do_range, ip_from, ip_to) = parse_ip_input(&ip_from_string, args.ip_to);

    // Create a ping client
    let config = Config::default();
    let client: Arc<Client> = Arc::new(Client::new(&config).unwrap());    
    
    if !do_range {
        println!("Scanning single IP {:?}", ip_from);
        
        let success_ping = ping_host_surge(&client, ip_from, timeout, args.verboose).await;
        println!("Status ping {:?} : {:?}", ip_from, success_ping);

        // // Test serializing and deserializing
        // let serialized = serde_json::to_string(&success_ping).unwrap();
        // println!("Serialized: {:?}", serialized);
        // let deserialized: network::network_core::PortScanResult = serde_json::from_str(&serialized).unwrap();
        // println!("Deserialized: {:?}", deserialized);

    } else {

        let ip_to = ip_to.expect("If we want to scan a range, ip_to must be supplied");
        
        println!("Scanning IP Range {:?} to {:?}", ip_from.to_string(), ip_to.to_string());

        // Get all IPs
        let ipv4_range: Vec<Ipv4Addr> = create_ipv4_range(ip_from, ip_to);
        let n_ips = ipv4_range.len() as u32;
        
        let progress_bar = Arc::new(Mutex::new(ProgressBar::new(n_ips as u64)));
        progress_bar.lock().unwrap().set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({eta})")
                .expect("Invalid template format"),
        );

    
        // Run concurrently
        let results = Arc::new(Mutex::new(Vec::new()));
        stream::iter(ipv4_range)
            .chunks(chunksize)
            .for_each_concurrent(None, |chunk| {
                let client_clone = Arc::clone(&client);
                let results = Arc::clone(&results);
                let pb = Arc::clone(&progress_bar);
                async move {
                    let mut local_results = Vec::new();
                    for ip in chunk {
                        let local_ping_result = ping_host_surge(&client_clone, ip, timeout, args.verboose).await;
                        local_results.push(local_ping_result);
                    }
                    let mut results = results.lock().unwrap();
                    results.extend(local_results);

                    // Increase the progressbar
                    let pb = pb.lock().unwrap();
                    pb.inc(1);
                }
            }).await;
        let results = results.lock().unwrap();
        
        // Gather information about how many IP scanned, how many are up etc
        let n_total: u32 = results.len() as u32;
        let mut n_up: u32 = 0;
        results.iter().for_each(|result| {
            if result.status == network::network_core::Status::Up {
                n_up += 1;
            }
        });

        // Print the results
        print_results(&results, n_total, n_up);
    }


    
    // // Test ping
    // let ip: IpAddr = "1.1.1.1".parse().unwrap();
    // let ip: IpAddr = "8.8.8.8".parse().unwrap();
    // let ip: IpAddr = "198.252.206.16".parse().unwrap(); // Stackoverflow
    // // let ip: IpAddr = "1.2.3.4".parse().unwrap();
    // println!("Pinging host: {:?}", ip);
    // let timeout: u32 = 100;
    // println!("Timeout: {:?}", timeout);
    // let success_ping = ping_host_syscmd(ip, timeout, true).await;
    // println!("Status ping {:?} : {:?}", ip, success_ping);


    // Test crate surge_ping
    // let timeout: u32 = 100;
    // let ip: IpAddr = "0.0.0.0".parse().unwrap(); // ERROR
    // let ip: IpAddr = "192.168.0.14".parse().unwrap(); // ERROR
    // let ip: IpAddr = "198.252.206.16".parse().unwrap(); // Stackoverflow
    // let ipnet: Ipv4Net = "192.168.0.0/28".parse().unwrap(); // Range
    // let ips: Vec<Ipv4Addr> = ipnet.hosts().collect();
    
    // let results = Arc::new(Mutex::new(Vec::new()));
    // let config = Config::default();
    // let client: Arc<Client> = Arc::new(Client::new(&config).unwrap());
    
    // stream::iter(ips)
    //     .chunks(10)
    //     .for_each_concurrent(None, |chunk| {
    //         let client_clone = Arc::clone(&client);
    //         let results = Arc::clone(&results);
    //         async move {
    //             let mut local_results = Vec::new();
    //             for ip in chunk {
    //                 println!("Pinging host: {:?}", ip);
    //                 let result = ping_host_surge(&client_clone, IpAddr::V4(ip), timeout).await;
    //                 let local_ping_result = match result {
    //                     Ok((_packet, _duration)) => {1},
    //                     Err(_) => {0}
    //                 };
    //                 local_results.push(local_ping_result);
    //             }
    //             let mut results = results.lock().unwrap();
    //             results.extend(local_results);
    //         }
    //     }).await;
    // let results = results.lock().unwrap();
    // println!("Results: {:?}", *results);

    // for ip in ips {

    //     println!("Pinging host: {:?}", ip);
    //     let result = ping_surge(&client, IpAddr::V4(ip)).await;
    //     match result {
    //         Ok((packet, duration)) => {
    //             // println!("Success: {:?}", packet);
    //             // println!("Duration: {:?}", duration);
    //             // // Extract the IP from the packet
    //             // match packet {
    //             //     IcmpPacket::V4(packet) => {
    //             //         let ip = packet.get_source();
    //             //         println!("Source IP: {:?}", ip);
    //             //     },
    //             //     _ => {
    //             //         println!("Not an Icmpv4Packet");
    //             //     }
    //             // }
    //         },
    //         Err(_) => {
    //             // println!("Ping not successful");

    //         }
    //     }
    // }

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