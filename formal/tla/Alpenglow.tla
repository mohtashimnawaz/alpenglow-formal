---- MODULE Alpenglow ----
EXTENDS Naturals, Sequences, TLC

\* Alpenglow Consensus Protocol (TLA+ Skeleton)
\* This is a high-level starting point for modeling Votor and Rotor

CONSTANTS NODES, SLOTS, STAKE, BYZANTINE, CRASHED

VARIABLES \* System state variables
    ledger, \* Sequence of finalized blocks
    votes,  \* [node -> [slot -> vote]]
    certificates, \* [slot -> certificate]
    leader, \* [slot -> node]
    timeouts, \* [node -> [slot -> timeout]]
    status \* [node -> status]

    skipCerts, \* [slot -> certificate] for skip logic
    rotorBlocks, \* [slot -> [fragment -> blockFragment]]
    relaySample \* [slot -> [node -> SUBSET Node]]

\* Types
Node == 1..NODES
Slot == 1..SLOTS
Stake == [Node -> Nat]

Fragment == 1..NumFragments
BlockFragment == Nat \* placeholder for erasure-coded fragment
NumFragments == 8 \* Example: 8-way erasure coding

\* Status: "honest", "byzantine", "crashed"
Status == {"honest", "byzantine", "crashed"}

\* Vote: (node, slot, block, path)
Vote == [node: Node, slot: Slot, block: Nat, path: {"fast", "slow"}]

\* Certificate: set of votes
Certificate == SUBSET Vote

\* Initial State
Init == 
    /\ ledger = << >>
    /\ votes = [n \in Node |-> [s \in Slot |-> {}]]
    /\ certificates = [s \in Slot |-> {}]
    /\ leader = [s \in Slot |-> 1]
    /\ timeouts = [n \in Node |-> [s \in Slot |-> 0]]
    /\ status = [n \in Node |-> "honest"]

    /\ skipCerts = [s \in Slot |-> {}]
    /\ rotorBlocks = [s \in Slot |-> [f \in Fragment |-> 0]]
    /\ relaySample = [s \in Slot |-> [n \in Node |-> {}]]

\* Actions (to be refined)
VoteAction == \E n \in Node, s \in Slot, b \in Nat, p \in {"fast", "slow"}:
    /\ status[n] = "honest"
    /\ votes[n][s] = votes[n][s] \cup {[node |-> n, slot |-> s, block |-> b, path |-> p]}

TimeoutAction == \E n \in Node, s \in Slot:
    /\ status[n] = "honest"
    /\ timeouts[n][s] = timeouts[n][s] + 1

SkipCertAction == \E s \in Slot:
    /\ \E skipcert \in SUBSET {v \in UNION {votes[n][s] : n \in Node}}:
        /\ Cardinality(skipcert) >= SlowQuorum
        /\ skipCerts[s] = skipcert

\* Rotor: Erasure-coded block propagation
RotorPropagate == \E s \in Slot, f \in Fragment, n \in Node:
    /\ status[n] = "honest"
    /\ \E relaySet \in SUBSET Node:
        /\ Cardinality(relaySet) >= RelaySampleSize
        /\ relaySample[s][n] = relaySet
    /\ rotorBlocks[s][f] = b \* b: block fragment value

CertifyAction == \E s \in Slot:
    /\ \E cert \in SUBSET {v \in UNION {votes[n][s] : n \in Node}}:
        /\ Cardinality(cert) >= FastQuorum \/ Cardinality(cert) >= SlowQuorum
        /\ certificates[s] = cert

\* Leader rotation (windowed)
LeaderRotate == \E s \in Slot:
    /\ leader[s] = ((s - 1) % NODES) + 1

\* Quorum thresholds
FastQuorum == (80 * NODES) \div 100
SlowQuorum == (60 * NODES) \div 100
RelaySampleSize == (20 * NODES) \div 100

\* Next-state relation
Next == VoteAction \/ CertifyAction \/ TimeoutAction \/ SkipCertAction \/ RotorPropagate \/ LeaderRotate

\* SAFETY PROPERTIES
\* No two conflicting blocks can be finalized in the same slot
Safety == \A s1, s2 \in Slot: s1 # s2 => certificates[s1] \cap certificates[s2] = {}

\* Chain consistency under up to 20% Byzantine stake
ChainConsistency == \A s \in Slot: 
    /\ certificates[s] # {} => 
        /\ \A v1, v2 \in certificates[s]: v1.block = v2.block
        /\ Cardinality({n \in Node : status[n] = "byzantine"}) <= (20 * NODES) \div 100

\* Certificate uniqueness and non-equivocation
CertificateUniqueness == \A s \in Slot, n \in Node:
    /\ Cardinality({v \in votes[n][s] : TRUE}) <= 1 \* No equivocation
    /\ certificates[s] # {} => 
        \A v \in certificates[s]: v.node # n \/ v \in votes[n][s]

\* LIVENESS PROPERTIES (temporal)
\* Progress guarantee under partial synchrony with >60% honest participation
ProgressGuarantee == <>(\A s \in Slot: certificates[s] # {} \/ skipCerts[s] # {})

\* Fast path completion in one round with >80% responsive stake
FastPathLiveness == 
    \A s \in Slot: 
        (Cardinality({n \in Node : status[n] = "honest"}) >= FastQuorum) 
        => <>(certificates[s] # {})

\* Bounded finalization time
BoundedFinalization == \A s \in Slot: 
    <>(certificates[s] # {} \/ skipCerts[s] # {})

\* RESILIENCE PROPERTIES
\* Safety maintained with ≤20% Byzantine stake
ByzantineResilience == 
    (Cardinality({n \in Node : status[n] = "byzantine"}) <= (20 * NODES) \div 100) 
    => Safety

\* Liveness maintained with ≤20% non-responsive stake
ResponsivenessResilience == 
    (Cardinality({n \in Node : status[n] = "crashed"}) <= (20 * NODES) \div 100) 
    => ProgressGuarantee

\* Network partition recovery guarantees
PartitionRecovery == \A s \in Slot:
    (skipCerts[s] # {}) => <>(certificates[s] # {})

\* TIMEOUT AND ROTOR INVARIANTS
\* Timeout/skip: If timeout threshold reached, skipCert must be present
TimeoutSkipInvariant == \A s \in Slot: 
    (\E n \in Node: timeouts[n][s] >= TimeoutThreshold) => skipCerts[s] # {}

\* Rotor: Each block fragment must be relayed by a sufficient sample
RotorRelayInvariant == \A s \in Slot, f \in Fragment: 
    (rotorBlocks[s][f] # 0) => 
        \E n \in Node: Cardinality(relaySample[s][n]) >= RelaySampleSize

\* Rotor: Erasure coding recovery guarantee
RotorRecoveryInvariant == \A s \in Slot:
    (Cardinality({f \in Fragment : rotorBlocks[s][f] # 0}) >= NumFragments \div 2)
    => \E b \in Nat: \A f \in Fragment: rotorBlocks[s][f] = b \/ rotorBlocks[s][f] = 0

TimeoutThreshold == 2 \* Example: 2 rounds

\* Specification
Spec == Init /\ [][Next]_<<ledger, votes, certificates, leader, timeouts, status, skipCerts, rotorBlocks, relaySample>>

\* Model checking configuration (for small node sets)
THEOREM SafetyTheorem == Spec => []Safety
THEOREM ChainConsistencyTheorem == Spec => []ChainConsistency  
THEOREM CertificateUniquenessTheorem == Spec => []CertificateUniqueness
THEOREM ByzantineResilienceTheorem == Spec => []ByzantineResilience
THEOREM TimeoutSkipTheorem == Spec => []TimeoutSkipInvariant
THEOREM RotorRelayTheorem == Spec => []RotorRelayInvariant
THEOREM RotorRecoveryTheorem == Spec => []RotorRecoveryInvariant

\* Temporal property theorems
THEOREM ProgressTheorem == Spec => ProgressGuarantee
THEOREM FastPathTheorem == Spec => FastPathLiveness
THEOREM BoundedFinalizationTheorem == Spec => BoundedFinalization
THEOREM ResponsivenessTheorem == Spec => ResponsivenessResilience
THEOREM PartitionRecoveryTheorem == Spec => PartitionRecovery

====