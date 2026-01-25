# 🌟 GlowBarn

**High-Performance Paranormal Detection Suite**

A cross-platform native application for paranormal investigation, environmental monitoring, and multi-modal anomaly detection. Built in Rust for maximum performance and safety.

![Version](https://img.shields.io/badge/version-2.0.0-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Platform](https://img.shields.io/badge/platform-Linux%20|%20macOS%20|%20Windows-lightgrey)

## ✨ Features

### 🔬 50+ Sensor Types
- **Thermal**: IR arrays, FLIR cameras, thermocouples
- **Electromagnetic**: EMF probes, flux gates, gaussmeters
- **Audio**: Infrasound, ultrasonic, EVP detection
- **Radiation**: Geiger counters, scintillators
- **Environmental**: Barometers, hygrometers, air quality
- **Quantum**: QRNG, entanglement detectors
- **Optical**: Spectrometers, laser grids, UV sensors

### 📊 Advanced Analysis
- **10+ Entropy Measures**: Shannon, Rényi, Tsallis, Approximate, Sample, Permutation
- **Anomaly Detection**: Z-score, MAD, CUSUM, Isolation Forest, Local Outlier Factor
- **Signal Processing**: FFT, wavelets, cross-correlation
- **Pattern Recognition**: Recurrence analysis, complexity measures

### 🎯 Multi-Sensor Fusion
- Bayesian fusion with confidence weighting
- Dempster-Shafer evidence theory
- Cross-sensor correlation analysis
- Temporal pattern detection

### 🔐 Security
- AES-256-GCM encryption
- ChaCha20-Poly1305 support
- Argon2id key derivation
- Secure memory handling (zeroize)

### 🖥️ Visual Console
- Real-time waveform displays
- Thermal heatmaps
- Spectrum analyzers
- Detection alerts
- Dark/Light themes

## 🚀 Quick Start

### Pre-built Binaries

Download from [Releases](https://github.com/bad-antics/glowbarn-rs/releases).

### Build from Source

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/bad-antics/glowbarn-rs.git
cd glowbarn-rs
cargo build --release --features gui

# Run
./target/release/glowbarn --demo
```

### System Dependencies (Linux)

```bash
sudo apt install libasound2-dev libudev-dev pkg-config libssl-dev \
  libxkbcommon-dev libwayland-dev
```

## 📖 Usage

```bash
# GUI mode with demo sensors
glowbarn --demo

# Headless server mode
glowbarn --headless --ws-port 8765

# With debug logging
glowbarn --demo --debug

# Custom config file
glowbarn -c /path/to/config.toml
```

## 🏗️ Architecture

```
glowbarn-rs/
├── src/
│   ├── core/          # Engine, event bus, scheduler
│   ├── sensors/       # 50+ sensor implementations
│   ├── analysis/      # Entropy, anomaly, signal processing
│   ├── detection/     # Fusion, classification, correlation
│   ├── security/      # Encryption, auth, secure memory
│   ├── streaming/     # MQTT, WebSocket, export
│   ├── gpu/           # wgpu compute shaders
│   ├── ui/            # egui visual console
│   ├── config/        # Configuration management
│   └── db/            # SQLite persistence
```

## 🔧 Configuration

Default config location: `~/.config/glowbarn/config.toml`

```toml
[sensors]
sample_rate = 1000.0
buffer_size = 4096

[analysis]
entropy_window = 256
anomaly_threshold = 3.0

[detection]
fusion_method = "bayesian"
min_confidence = 0.7

[gui]
theme = "dark"
refresh_rate = 60
```

## 📡 Streaming

### WebSocket API

Connect to `ws://localhost:8765` for real-time data:

```json
{
  "type": "reading",
  "sensor_id": "emf-probe-1",
  "timestamp": "2026-01-24T12:00:00Z",
  "values": [0.5, 0.7, 0.3],
  "quality": 0.95
}
```

### MQTT

Publish to topics:
- `glowbarn/readings/{sensor_id}`
- `glowbarn/detections`
- `glowbarn/alerts`

## 🤝 Contributing

Contributions welcome! Please read our [Contributing Guide](CONTRIBUTING.md).

## 📜 License

MIT License - see [LICENSE](LICENSE) for details.

## 🔗 Related Projects

- [glowbarn-os](https://github.com/bad-antics/glowbarn-os) - Custom Linux OS for GlowBarn
- [glowbarn](https://github.com/bad-antics/glowbarn) - Original Python prototype

---

**Made with 🔮 by [bad-antics](https://github.com/bad-antics)**
