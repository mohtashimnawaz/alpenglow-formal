# Alpenglow Formal Verification

This directory contains the TLA+ specification for the Alpenglow consensus protocol, implementing formal verification of Votor's dual-path consensus and Rotor's erasure-coded block propagation.

## Files
- `Alpenglow.tla`: Complete TLA+ model with Votor, Rotor, certificates, timeouts, skip logic, and leader rotation
- `AlpenglowModel.cfg`: TLC configuration file with constants and properties to check
- `README.md`: This documentation file

## Model Overview

### Protocol Components Modeled
1. **Votor Dual-Path Consensus**: Fast path (80% stake) and slow path (60% stake) finalization
2. **Rotor Block Propagation**: Erasure-coded fragments with stake-weighted relay sampling
3. **Certificate Generation**: Vote aggregation and quorum validation
4. **Timeout Mechanisms**: Skip certificate logic for liveness under network delays
5. **Leader Rotation**: Windowed leader selection across slots

### Properties Verified

#### Safety Properties
- **No Conflicting Finalization**: No two different blocks finalized in same slot
- **Chain Consistency**: Consistent block selection under ≤20% Byzantine stake
- **Certificate Uniqueness**: Non-equivocation and unique vote constraints

#### Liveness Properties  
- **Progress Guarantee**: Eventually some certificate or skip certificate in every slot
- **Fast Path Completion**: One-round finalization with >80% responsive stake
- **Bounded Finalization**: Finite-time finalization guarantee

#### Resilience Properties
- **Byzantine Resilience**: Safety maintained with ≤20% Byzantine stake
- **Responsiveness Resilience**: Liveness maintained with ≤20% crashed nodes
- **Partition Recovery**: Network partition recovery through skip certificates

## Running Model Checking

### Prerequisites
Install [TLA+ Toolbox](https://lamport.azurewebsites.net/tla/toolbox.html) or TLC command line tools.

### Small Configuration Testing (4 nodes)
```bash
tlc -config AlpenglowModel.cfg Alpenglow.tla
```

### Larger Configurations
Edit `AlpenglowModel.cfg` to increase `NODES` (e.g., 6-10 nodes):
```
NODES = 6
SLOTS = 5
```

### Expected Results
- **Safety invariants**: Should hold for all reachable states
- **Liveness properties**: May require fairness assumptions or bounded model checking
- **Resilience bounds**: Verify tolerance levels match theoretical bounds (20%+20%)

## Current Proof Status

| Property | Status | Notes |
|----------|---------|-------|
| Safety | ✓ Modeled | Basic conflict-free finalization |
| Chain Consistency | ✓ Modeled | Byzantine tolerance formalized |
| Certificate Uniqueness | ✓ Modeled | Anti-equivocation constraints |
| Progress Guarantee | ✓ Modeled | Requires fairness assumptions |
| Fast Path Liveness | ✓ Modeled | 80% threshold conditions |
| Byzantine Resilience | ✓ Modeled | 20% Byzantine bound |
| Rotor Recovery | ✓ Modeled | Erasure coding invariants |

## Next Steps

### Model Refinement
1. **Stake Weighting**: Replace uniform node model with realistic stake distribution
2. **Network Delays**: Add explicit timing and partial synchrony assumptions  
3. **Fairness Constraints**: Add fairness assumptions for liveness properties
4. **Byzantine Behaviors**: Model specific Byzantine attack patterns

### Verification Scale-Up
1. **Proof Decomposition**: Break complex properties into provable lemmas
2. **Inductive Invariants**: Strengthen invariants for automated verification
3. **Bounded Model Checking**: Use time bounds for liveness verification
4. **Statistical Model Checking**: Scale to realistic network sizes (100+ nodes)

### Integration Testing
1. **Reference Implementation**: Cross-validate with Solana's Alpenglow code
2. **Performance Analysis**: Verify finalization time bounds (100-150ms)
3. **Attack Scenarios**: Test against known consensus attacks

## Limitations

- **Abstract Model**: Network timing and cryptographic details abstracted
- **Small State Space**: Current configuration limited to ~10 nodes for exhaustive checking
- **Simplified Erasure Coding**: Rotor fragments modeled as integers, not actual Reed-Solomon codes
- **Missing Economics**: Stake slashing and economic incentives not modeled

## Contributing

To extend this model:
1. Preserve existing invariants when adding new actions
2. Add new properties as both invariants (safety) and temporal formulas (liveness)
3. Test with small configurations before scaling up
4. Document any assumptions or simplifications made
