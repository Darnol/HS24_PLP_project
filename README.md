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