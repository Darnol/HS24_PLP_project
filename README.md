# HS24_PLP_project
HS24 Paradigms of Programming Languages Project

# Windows
netowkring is fuuuuucked.
see  
https://github.com/libpnet/libpnet/issues/332

putting the Package.lib and wpcap.lib into the .rustup toolchain as proposed
--> Does not really work


# What does it do?
- Analyses all the available interfaces at start
- Uses the system command `ping` to check if a host is live
- Uses `TcpStream` to check some common TCP ports
- Uses `dns_lookup` to check if DNS resolution can be done to get a hostname


# Run the docker container to show nmap
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


# How to demonstrate
1. `cargo run` - Determine my subnet and show the interfaces
2. `cargo run 10.28.207.15/26` - Show how it scans the subnet, although this is a rather boring result
3. `nmap -v 10.28.207.15/26` - To show what namp is capable of



# Take away
- Very different for variying OS. Windwos vs Macos
    - Macos hostname resolution is really not working
    - Works on Windows at home in the home network
    - Some crates did not work on Windows, only implemented for Linux/Unix
- There are many different crates and possibilities to implement networking
- There are many many badly maintained crates I think
- Especially the concurrency can be implemented in like 10 different ways