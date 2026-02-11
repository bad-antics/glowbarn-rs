# GlowBarn-RS Documentation

## Overview

GlowBarn-RS is the Rust implementation of the GlowBarn IoT security platform. It provides tools for IoT device discovery, vulnerability assessment, and firmware analysis with the performance and safety guarantees of Rust.

## Features

- **Device Discovery** — UPnP, mDNS, and custom protocol scanning
- **Firmware Analysis** — Binary extraction, string analysis, crypto key detection
- **Protocol Fuzzing** — Smart fuzzing for MQTT, CoAP, and custom IoT protocols
- **Vulnerability Scanner** — Check for default credentials, known CVEs, misconfigurations
- **Network Mapping** — Visualize IoT device relationships and communication patterns

## Architecture

```rust
// Module structure
pub mod discovery {
    pub mod upnp;       // UPnP SSDP discovery
    pub mod mdns;       // mDNS/Bonjour scanning
    pub mod arp;        // ARP-based discovery
    pub mod custom;     // Custom protocol probes
}

pub mod analysis {
    pub mod firmware;   // Firmware extraction
    pub mod protocol;   // Protocol analysis
    pub mod vulns;      // Vulnerability checks
}

pub mod fuzzing {
    pub mod mqtt;       // MQTT fuzzing
    pub mod coap;       // CoAP fuzzing
    pub mod generic;    // Generic TCP/UDP fuzzing
}
```

## Usage

```bash
# Discover IoT devices on network
glowbarn-rs discover --network 192.168.1.0/24

# Analyze firmware image
glowbarn-rs firmware analyze --input firmware.bin

# Fuzz MQTT broker
glowbarn-rs fuzz mqtt --target broker.local:1883
```

## Related

- [GlowBarn OS](https://github.com/bad-antics/glowbarn-os) — IoT security operating system
