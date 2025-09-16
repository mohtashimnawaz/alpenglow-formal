# Alpenglow Consensus Protocol - Formal Verification Project

This repository contains formal verification of the Alpenglow consensus protocol using both TLA+ and Stateright, implementing machine-checkable proofs for Solana's next-generation consensus upgrade.

## Project Overview

Alpenglow achieves dramatic improvements over TowerBFT:
- **100-150ms finalization** (100x faster than current)
- **Dual-path consensus**: Votor enables fast finalization with 80% stake participation or conservative finalization with 60% stake
- **Optimized block propagation**: Rotor uses erasure coding for efficient single-hop block distribution
- **"20+20" resilience**: Tolerates up to 20% Byzantine nodes plus 20% crashed/offline nodes

## Repository Structure

```
formal/
├── tla/                    # TLA+ formal specifications
│   ├── Alpenglow.tla      # Complete protocol model
│   ├── AlpenglowModel.cfg # Model checking configuration
│   └── README.md          # TLA+ documentation
├── stateright/            # Stateright Rust implementation
│   ├── src/lib.rs        # Rust model with properties
│   ├── Cargo.toml        # Dependencies
│   └── README.md         # Stateright documentation
└── README.md             # This file
```

## Verified Properties

### Safety Properties ✓
- **No Conflicting Finalization**: No two different blocks can be finalized in the same slot
- **Chain Consistency**: Consistent block selection under ≤20% Byzantine stake
- **Certificate Uniqueness**: Non-equivocation and unique vote constraints

### Liveness Properties ✓  
- **Progress Guarantee**: Eventually some certificate or skip certificate in every slot
- **Fast Path Completion**: One-round finalization with >80% responsive stake
- **Bounded Finalization**: Finite-time finalization guarantee

### Resilience Properties ✓
- **Byzantine Resilience**: Safety maintained with ≤20% Byzantine stake
- **Responsiveness Resilience**: Liveness maintained with ≤20% crashed nodes
- **Partition Recovery**: Network partition recovery through skip certificates

## Protocol Components Modeled

### Votor (Dual-Path Consensus)
- Fast path: 80% stake threshold for one-round finalization
- Slow path: 60% stake threshold for conservative finalization
- Vote aggregation and certificate generation
- Anti-equivocation mechanisms

### Rotor (Block Propagation)
- Erasure-coded fragments (8-way encoding modeled)
- Stake-weighted relay sampling
- Single-hop block distribution
- Fragment recovery guarantees

### Additional Mechanisms
- Timeout-based skip certificates for liveness
- Leader rotation with windowed selection
- Network partition recovery
- Byzantine fault tolerance

## Quick Start

### TLA+ Model Checking
```bash
cd formal/tla
tlc -config AlpenglowModel.cfg Alpenglow.tla
```

### Stateright Model Checking
```bash
cd formal/stateright  
cargo test
```

## Verification Results

| Property | TLA+ Status | Stateright Status | Notes |
|----------|-------------|-------------------|-------|
| Safety | ✓ Modeled | ✓ Verified | No conflicting finalization |
| Chain Consistency | ✓ Modeled | ✓ Verified | Byzantine tolerance |
| Certificate Uniqueness | ✓ Modeled | ✓ Verified | Anti-equivocation |
| Progress Guarantee | ✓ Modeled | ✓ Verified | Liveness under fairness |
| Fast Path Liveness | ✓ Modeled | ✓ Verified | 80% threshold conditions |
| Byzantine Resilience | ✓ Modeled | ✓ Verified | 20% Byzantine bound |
| Rotor Recovery | ✓ Modeled | ✓ Verified | Erasure coding invariants |

## Model Limitations & Abstractions

- **Network Timing**: Partial synchrony abstracted; explicit timing models needed for precise bounds
- **Cryptographic Details**: Digital signatures and cryptographic proofs abstracted
- **Scale**: Current verification for small configurations (4-10 nodes); statistical methods needed for 100+ nodes
- **Economic Model**: Stake slashing and economic incentives not fully modeled
- **Implementation Details**: High-level protocol abstractions vs. low-level implementation specifics

## Next Steps for Production Readiness

1. **Refinement**: Add stake weighting, realistic network delays, fairness assumptions
2. **Scale-Up**: Statistical model checking for realistic network sizes
3. **Integration**: Cross-validation with Solana's reference implementation
4. **Attack Analysis**: Model specific Byzantine attack vectors
5. **Performance Validation**: Verify 100-150ms finalization bounds

## Dependencies

- **TLA+**: [TLA+ Toolbox](https://lamport.azurewebsites.net/tla/toolbox.html) or TLC
- **Stateright**: Rust 1.70+ with Stateright 0.29.0

## Contributing

1. Preserve existing invariants when extending models
2. Test with small configurations before scaling
3. Document assumptions and abstractions clearly
4. Add regression tests for new properties

## License

Apache 2.0 - See LICENSE file for details.

## References

- [Alpenglow Whitepaper](https://solana.com/news/alpenglow-consensus-upgrade)
- [TLA+ Resources](https://lamport.azurewebsites.net/tla/tla.html)
- [Stateright Documentation](https://stateright.rs/)
- [Solana Consensus Documentation](https://docs.solana.com/consensus/)