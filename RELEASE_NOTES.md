# GlowBarn v2.0.0 - Rust Edition 🌟

**High-Performance Paranormal Detection Suite**

The complete rewrite of GlowBarn in Rust, delivering 10-100x performance improvements over the Python prototype.

## ✨ Highlights

- **~14,000 lines of Rust** - Complete ground-up rewrite
- **50+ sensor types** - From EMF to quantum random number generators
- **Native cross-platform GUI** - egui-based visual console
- **Military-grade encryption** - AES-256-GCM, Argon2id
- **Real-time streaming** - WebSocket & MQTT support

## 🆕 What's New

### Core Engine
- Async Rust architecture with Tokio
- Lock-free event bus for sensor data
- Parallel analysis with Rayon

### Sensors (50+ types)
- Thermal: IR arrays, FLIR, thermocouples
- EMF: Probes, flux gates, gaussmeters  
- Audio: Infrasound, ultrasonic, EVP
- Radiation: Geiger, scintillator
- Quantum: QRNG, entanglement detection
- And many more...

### Analysis
- 10+ entropy measures (Shannon, Rényi, Tsallis, etc.)
- 5+ anomaly detection algorithms
- FFT, wavelets, cross-correlation
- Complexity measures (Lyapunov, Hurst)

### Detection
- Bayesian sensor fusion
- Dempster-Shafer evidence theory
- Cross-sensor correlation
- Classification system

### Security
- AES-256-GCM & ChaCha20-Poly1305 encryption
- Argon2id key derivation
- Secure memory with zeroize
- Session-based authentication

### Visual Console
- Real-time waveforms
- Thermal heatmaps
- Spectrum analyzers
- Detection alerts
- Dark/Light themes

## 📦 Downloads

| Platform | Download |
|----------|----------|
| Linux x86_64 | [glowbarn-linux-x86_64](https://github.com/bad-antics/glowbarn-rs/releases/download/v2.0.0/glowbarn-linux-x86_64) |

## 🚀 Quick Start

```bash
# Download and run
chmod +x glowbarn-linux-x86_64
./glowbarn-linux-x86_64 --demo

# Or build from source
git clone https://github.com/bad-antics/glowbarn-rs.git
cd glowbarn-rs
cargo build --release --features gui
./target/release/glowbarn --demo
```

## 📋 System Requirements

### Linux
```bash
sudo apt install libasound2-dev libudev-dev pkg-config libssl-dev
```

## 🔮 Coming Soon

- macOS and Windows builds
- GPU compute acceleration
- Machine learning classification
- Mobile companion app

---

**Full Changelog**: https://github.com/bad-antics/glowbarn-rs/commits/v2.0.0
