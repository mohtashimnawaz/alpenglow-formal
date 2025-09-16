# Alpenglow Stateright Model

This directory contains the Stateright implementation of the Alpenglow consensus protocol for model checking and verification.

## Files
- `src/lib.rs`: Complete Stateright model with properties
- `Cargo.toml`: Rust project configuration
- `README.md`: This documentation

## Running Model Checking

### Prerequisites
Install Rust and Cargo:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Run Tests and Model Checking
```bash
cd /Users/mohtashimnawaz/Desktop/alpenglow-formal/formal/stateright
cargo test
```

### Interactive Model Explorer
```bash
cargo run --example explorer
```

## Model Overview

The Stateright model implements:

1. **State Structure**: Nodes, votes, certificates, timeouts, Rotor fragments
2. **Actions**: Vote, Certify, Timeout, SkipCert, RotorPropagate, LeaderRotate  
3. **Properties**: Safety, chain consistency, Byzantine resilience, progress

### Verified Properties
- **Safety**: No conflicting certificates in different slots
- **Chain Consistency**: All votes in certificate for same block
- **Byzantine Resilience**: Tolerates ≤20% Byzantine nodes
- **Progress**: Eventually certificates or skip certificates exist

## Next Steps
- Add more sophisticated Byzantine behaviors
- Implement realistic network delays and message ordering
- Scale to larger node sets with statistical model checking
- Add economic incentives and slashing conditions