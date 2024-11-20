# Use the latest Ubuntu image
FROM ubuntu:latest

# Set the environment variable to avoid prompts during installation
ENV DEBIAN_FRONTEND=noninteractive

# Update the package list and install required packages
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    nmap \
    iputils-ping \
    iproute2 && \
    # Clean up to reduce image size
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Set the default command to bash
CMD ["bash"]
