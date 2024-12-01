# Concurrent high-performance network scanner
HS24 Paradigms of Programming Languages - Project  
Author: Dominik Arnold (dominikjohann.arnold@uzh.ch)

# How to run it?

## Install Rust
To get started, you'll have to install Rust, i.e. the compiler `rustc` and the package (called 'crate' in Rust) management software `cargo`.  
There are excellent instructions on the official Rust homepage: https://www.rust-lang.org/tools/install  
There you can download the `rustup` tool that will guide you through the installation process.  
After the installation, check if the compiler and cargo is installed by opening a terminal and executing
```bash
rustc --version
cargo --version
```
## Run the program
From a terminal in the root run `cargo build` to install all neccesary dependencies.  
Then run `cargo run -- --help` to display the help page of the CLI tool. Some common use cases:
- `cargo run` - Only run the network interface analysis
- `cargo run 192.168.0.1` - Scan a single IPv4 address
- `cargo run 192.168.0.0/24` - Scan a range given by CIDR notation, in this case hosts from 192.168.0.1 to 192.168.0.254
- `cargo run 192.168.0.1 192.168.0.10` - Scan a range given by two IPv4 addresses, in this case from 192.168.0.1 to 192.168.0.10

# What does it do?
- Analyses all the available interfaces at start
- Accepts keywords to specify a single IPv4 address or an IPv4 address range
    - A single IPv4 address
    - Two IPv4 addresses specifying the start and end of the desired range
    - A CIDR notation [see here](https://de.wikipedia.org/wiki/Classless_Inter-Domain_Routing) specifying a range
- Given a IPv4 address or range, it will scan ever host:
    - Uses an ICMP ping command to check a hosts liveliness
    - Uses TCP socket to detect open TCP ports
    - Uses the OS DNS resolver to determine the human-readable hostname if available
- Uses Rusts concurrency features to scan the range of hosts as quickly as possible
- Will print a final report

# How to demonstrate the tool
- `cargo run -- --help` - Show the CLI help
- `cargo run` - Determine my subnet and show the interfaces
- `cargo run 10.28.207.15/27` - Show how it scans the subnet, although this is a rather boring result
- `cargo run 10.28.207.1 10.28.207.20` - Show an alternative way of specifying the IP range
- `nmap -v 10.28.207.15/27` - To show what namp is capable of

# Run the docker container to test nmap on Windows
If you're on a windows machine, you won't be able to test out the nmap tool, since it is written for unix systems. You can spin up a docker container and test the namp tool there:  

```
docker build -t nmap_test .
docker run -itd --name nmap_test_container nmap_test
docker exec -it nmap_test_container bash
```

Run some nmap commands:
```
nmap -v -F 192.168.0.1/24
-v verboose
-F fast (not 10000 ports per host)
```

# Take away
- There are many different crates and possibilities to implement networking
    - For example for a reverse DNS lookup, there are at least 4 crates that promise a solution, some more successful, some less. The result oftentimes depends on the used OS.
- There are many many badly maintained crates
- Very different for variying OS. Windwos vs Macos
    - Works on Windows at home in the home network
    - Some crates did not work on Windows, only implemented for Linux/Unix
- Especially the concurrency can be implemented in like 10 different ways