use stateright::*;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use serde::{Deserialize, Serialize};

/// Alpenglow Consensus Protocol - Enhanced Stateright Model
/// Implements Votor dual-path consensus with improved stake weighting and Byzantine behaviors

pub type NodeId = u32;
pub type Slot = u32;
pub type BlockId = u32;
pub type StakeAmount = u64;
pub type Timestamp = u64;
pub type Round = u32;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AlpenglowState {
    pub nodes: Vec<NodeId>,
    pub stake_distribution: HashMap<NodeId, StakeAmount>,
    pub current_slot: Slot,
    pub global_time: Timestamp,
    pub ledger: Vec<FinalizedBlock>,
    pub votes: HashMap<NodeId, HashMap<Slot, Vec<Vote>>>,
    pub certificates: HashMap<Slot, Certificate>,
    pub skip_certs: HashMap<Slot, SkipCertificate>,
    pub timeouts: HashMap<NodeId, HashMap<Slot, TimeoutInfo>>,
    pub status: HashMap<NodeId, NodeStatus>,
    pub network_partition: Option<NetworkPartition>,
    pub byzantine_coalitions: Vec<ByzantineCoalition>,
    pub coalition_state: HashMap<usize, CoalitionState>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FinalizedBlock {
    pub slot: Slot,
    pub block_id: BlockId,
    pub finalization_time: Timestamp,
    pub total_stake: StakeAmount,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimeoutInfo {
    pub count: u32,
    pub last_timeout: Timestamp,
    pub threshold: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NodeStatus {
    Honest,
    Byzantine(ByzantineStrategy),
    Crashed { since: Timestamp },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ByzantineStrategy {
    // Basic Byzantine behaviors
    Equivocation,    // Vote for multiple blocks
    WithholdVotes,   // Don't participate
    RandomVotes,     // Vote randomly
    
    // Advanced sophisticated Byzantine behaviors
    SelectiveEquivocation {
        /// Only equivocate when the node has high stake influence
        min_stake_threshold: StakeAmount,
        /// Target specific slots for maximum disruption
        target_slots: Vec<Slot>,
    },
    AdaptiveBehavior {
        /// Switch strategies based on network conditions
        primary_strategy: Box<ByzantineStrategy>,
        fallback_strategy: Box<ByzantineStrategy>,
        /// Trigger threshold (e.g., number of timeouts)
        adaptation_threshold: u32,
    },
    CoalitionAttack {
        /// Coordinate with other Byzantine nodes
        coalition_members: Vec<NodeId>,
        /// Attack strategy for the coalition
        attack_type: CoalitionAttackType,
    },
    TimingAttack {
        /// Delay votes to disrupt timing assumptions
        delay_votes: bool,
        /// Maximum delay in milliseconds
        max_delay: u64,
        /// Target path to attack (Fast/Slow)
        target_path: Option<VotePath>,
    },
    StakeBasedAttack {
        /// Use economic power to maximize damage
        reserve_stake_for_critical_slots: bool,
        /// Minimum stake to activate attack
        activation_threshold: StakeAmount,
        /// Economic incentive threshold
        min_profit_margin: StakeAmount,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CoalitionAttackType {
    /// All coalition members vote for different blocks to split the network
    SplitVote {
        target_blocks: Vec<BlockId>,
    },
    /// Coalition withholds votes until last moment then floods network
    DelayedFlood {
        delay_until_slot: Slot,
    },
    /// Coalition targets specific high-value slots for maximum disruption
    StrategicTargeting {
        high_priority_slots: Vec<Slot>,
        disruption_threshold: f64, // Minimum disruption probability
    },
    /// Coalition attempts to manipulate certificate generation
    CertificateManipulation {
        target_path: VotePath,
        manipulation_type: CertManipulationType,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CertManipulationType {
    /// Prevent certificate formation by withholding critical votes
    PreventCertification,
    /// Create conflicting certificates for same slot
    ConflictingCertificates,
    /// Delay certificate formation to cause timeouts
    DelayedCertification { delay_slots: u32 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct Vote {
    pub node: NodeId,
    pub slot: Slot,
    pub block: BlockId,
    pub path: VotePath,
    pub stake: StakeAmount,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub enum VotePath {
    Fast,  // 80% stake threshold
    Slow,  // 60% stake threshold
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Certificate {
    pub votes: HashSet<Vote>,
    pub slot: Slot,
    pub block: BlockId,
    pub total_stake: StakeAmount,
    pub path: VotePath,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SkipCertificate {
    pub slot: Slot,
    pub timeout_votes: HashSet<Vote>,
    pub total_stake: StakeAmount,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NetworkPartition {
    pub partition_a: HashSet<NodeId>,
    pub partition_b: HashSet<NodeId>,
    pub started_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ByzantineCoalition {
    pub members: Vec<NodeId>,
    pub strategy: CoalitionAttackType,
    pub coordination_history: Vec<CoordinationEvent>,
    pub total_stake: StakeAmount,
    pub formation_time: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CoordinationEvent {
    pub slot: Slot,
    pub event_type: EventType,
    pub participants: Vec<NodeId>,
    pub timestamp: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EventType {
    CoordinatedVote { target_block: BlockId },
    CoordinatedWithhold { target_slot: Slot },
    MessageFlood { message_count: u32 },
    TimingAttack { delay_ms: u64 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CoalitionState {
    pub active: bool,
    pub current_phase: AttackPhase,
    pub success_metrics: AttackMetrics,
    pub adaptation_count: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AttackPhase {
    Preparation,
    Execution,
    Completion,
    Adaptation,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AttackMetrics {
    pub slots_disrupted: u32,
    pub certificates_prevented: u32,
    pub timeouts_caused: u32,
    pub economic_damage: StakeAmount,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AlpenglowAction {
    Vote { node: NodeId, slot: Slot, block: BlockId, path: VotePath },
    ByzantineVote { node: NodeId, strategy: ByzantineStrategy, slot: Slot },
    Certify { slot: Slot, path: VotePath },
    Timeout { node: NodeId, slot: Slot },
    SkipCert { slot: Slot },
    AdvanceTime { delta: Timestamp },
    NetworkPartition { nodes_a: HashSet<NodeId>, nodes_b: HashSet<NodeId> },
    HealPartition,
    // Advanced coalition actions
    FormCoalition { members: Vec<NodeId>, strategy: CoalitionAttackType },
    CoordinateAttack { coalition_index: usize, target_slot: Slot },
    AdaptStrategy { node: NodeId, new_strategy: ByzantineStrategy, reason: String },
    TimingManipulation { node: NodeId, delay_ms: u64, target_slot: Slot },
}

impl AlpenglowState {
    pub fn new(nodes: Vec<NodeId>, stake_distribution: HashMap<NodeId, StakeAmount>) -> Self {
        let mut votes = HashMap::new();
        let mut timeouts = HashMap::new();
        let mut status = HashMap::new();
        
        for &node in &nodes {
            let mut node_votes = HashMap::new();
            let mut node_timeouts = HashMap::new();
            for slot in 1..=5 {
                node_votes.insert(slot, Vec::new());
                node_timeouts.insert(slot, TimeoutInfo {
                    count: 0,
                    last_timeout: 0,
                    threshold: 3,
                });
            }
            votes.insert(node, node_votes);
            timeouts.insert(node, node_timeouts);
            status.insert(node, NodeStatus::Honest);
        }
        
        Self {
            nodes,
            stake_distribution,
            current_slot: 1,
            global_time: 0,
            ledger: Vec::new(),
            votes,
            certificates: HashMap::new(),
            skip_certs: HashMap::new(),
            timeouts,
            status,
            network_partition: None,
            byzantine_coalitions: Vec::new(),
            coalition_state: HashMap::new(),
        }
    }
    
    pub fn total_stake(&self) -> StakeAmount {
        self.stake_distribution.values().sum()
    }
    
    pub fn fast_quorum_stake(&self) -> StakeAmount {
        (80 * self.total_stake()) / 100
    }
    
    pub fn slow_quorum_stake(&self) -> StakeAmount {
        (60 * self.total_stake()) / 100
    }
    
    pub fn byzantine_threshold_stake(&self) -> StakeAmount {
        (20 * self.total_stake()) / 100
    }
    
    pub fn honest_stake(&self) -> StakeAmount {
        self.stake_distribution.iter()
            .filter(|(&node, _)| matches!(self.status[&node], NodeStatus::Honest))
            .map(|(_, stake)| stake)
            .sum()
    }
    
    pub fn byzantine_stake(&self) -> StakeAmount {
        self.stake_distribution.iter()
            .filter(|(&node, _)| matches!(self.status[&node], NodeStatus::Byzantine(_)))
            .map(|(_, stake)| stake)
            .sum()
    }
    
    pub fn is_network_partitioned(&self) -> bool {
        self.network_partition.is_some()
    }
    
    pub fn can_node_communicate(&self, node1: NodeId, node2: NodeId) -> bool {
        match &self.network_partition {
            None => true,
            Some(partition) => {
                (partition.partition_a.contains(&node1) && partition.partition_a.contains(&node2)) ||
                (partition.partition_b.contains(&node1) && partition.partition_b.contains(&node2))
            }
        }
    }
}

// Custom Hash implementation for efficient state exploration
impl Hash for AlpenglowState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the essential state components for efficient exploration
        self.current_slot.hash(state);
        self.global_time.hash(state);
        
        // Hash the node count and their basic status
        self.nodes.len().hash(state);
        for &node in &self.nodes {
            node.hash(state);
            // Hash node status simplified
            match &self.status[&node] {
                NodeStatus::Honest => 0u8.hash(state),
                NodeStatus::Byzantine(strategy) => {
                    1u8.hash(state);
                    match strategy {
                        ByzantineStrategy::Equivocation => 0u8.hash(state),
                        ByzantineStrategy::WithholdVotes => 1u8.hash(state),
                        ByzantineStrategy::RandomVotes => 2u8.hash(state),
                        ByzantineStrategy::SelectiveEquivocation { .. } => 3u8.hash(state),
                        ByzantineStrategy::AdaptiveBehavior { .. } => 4u8.hash(state),
                        ByzantineStrategy::CoalitionAttack { .. } => 5u8.hash(state),
                        ByzantineStrategy::TimingAttack { .. } => 6u8.hash(state),
                        ByzantineStrategy::StakeBasedAttack { .. } => 7u8.hash(state),
                    }
                },
                NodeStatus::Crashed { since } => {
                    2u8.hash(state);
                    since.hash(state);
                }
            }
        }
        
        // Hash certificate and skip cert count (simplified)
        self.certificates.len().hash(state);
        self.skip_certs.len().hash(state);
        self.ledger.len().hash(state);
        
        // Hash partition status
        self.is_network_partitioned().hash(state);
        
        // Hash total vote count per slot (simplified to avoid deep hashing)
        for slot in 1..=self.current_slot {
            let total_votes: usize = self.votes.values()
                .map(|node_votes| node_votes.get(&slot).map_or(0, |v| v.len()))
                .sum();
            total_votes.hash(state);
        }
    }
}

impl Model for AlpenglowState {
    type State = AlpenglowState;
    type Action = AlpenglowAction;
    
    fn init_states(&self) -> Vec<Self::State> {
        vec![self.clone()]
    }
    
    fn actions(&self, state: &Self::State, actions: &mut Vec<Self::Action>) {
        // Time advancement
        actions.push(AlpenglowAction::AdvanceTime { delta: 1 });
        
        // Voting actions
        for &node in &state.nodes {
            match &state.status[&node] {
                NodeStatus::Honest => {
                    for slot in state.current_slot..=std::cmp::min(state.current_slot + 1, 5) {
                        for block in 0..2 {
                            actions.push(AlpenglowAction::Vote {
                                node, slot, block, path: VotePath::Fast
                            });
                            actions.push(AlpenglowAction::Vote {
                                node, slot, block, path: VotePath::Slow
                            });
                        }
                    }
                }
                NodeStatus::Byzantine(strategy) => {
                    for slot in state.current_slot..=std::cmp::min(state.current_slot + 1, 5) {
                        actions.push(AlpenglowAction::ByzantineVote {
                            node, strategy: strategy.clone(), slot
                        });
                    }
                }
                _ => {} // Crashed nodes don't act
            }
        }
        
        // Certificate generation
        for slot in 1..=state.current_slot {
            actions.push(AlpenglowAction::Certify { slot, path: VotePath::Fast });
            actions.push(AlpenglowAction::Certify { slot, path: VotePath::Slow });
        }
        
        // Timeout actions
        for &node in &state.nodes {
            if matches!(state.status[&node], NodeStatus::Honest) {
                for slot in 1..=state.current_slot {
                    actions.push(AlpenglowAction::Timeout { node, slot });
                }
            }
        }
        
        // Skip certificates
        for slot in 1..=state.current_slot {
            actions.push(AlpenglowAction::SkipCert { slot });
        }
        
        // Network partition scenarios
        if state.network_partition.is_none() && state.nodes.len() >= 4 {
            let mid = state.nodes.len() / 2;
            let partition_a: HashSet<_> = state.nodes[0..mid].iter().cloned().collect();
            let partition_b: HashSet<_> = state.nodes[mid..].iter().cloned().collect();
            actions.push(AlpenglowAction::NetworkPartition {
                nodes_a: partition_a,
                nodes_b: partition_b,
            });
        }
        
        if state.network_partition.is_some() {
            actions.push(AlpenglowAction::HealPartition);
        }
        
        // Coalition formation and coordination
        let byzantine_nodes: Vec<_> = state.nodes.iter()
            .filter(|&&node| matches!(state.status[&node], NodeStatus::Byzantine(_)))
            .cloned()
            .collect();
            
        if byzantine_nodes.len() >= 2 {
            // Form coalitions of different sizes
            for size in 2..=std::cmp::min(byzantine_nodes.len(), 4) {
                if byzantine_nodes.len() >= size {
                    let coalition_members = byzantine_nodes[0..size].to_vec();
                    
                    // Different coalition strategies
                    actions.push(AlpenglowAction::FormCoalition {
                        members: coalition_members.clone(),
                        strategy: CoalitionAttackType::SplitVote { 
                            target_blocks: vec![0, 1, 2] 
                        },
                    });
                    
                    actions.push(AlpenglowAction::FormCoalition {
                        members: coalition_members,
                        strategy: CoalitionAttackType::StrategicTargeting { 
                            high_priority_slots: vec![1, 3, 5],
                            disruption_threshold: 0.5,
                        },
                    });
                }
            }
        }
        
        // Coalition coordination actions
        for (coalition_index, coalition_state) in &state.coalition_state {
            if coalition_state.active {
                for slot in state.current_slot..=std::cmp::min(state.current_slot + 1, 5) {
                    actions.push(AlpenglowAction::CoordinateAttack {
                        coalition_index: *coalition_index,
                        target_slot: slot,
                    });
                }
            }
        }
        
        // Strategy adaptation for Byzantine nodes
        for &node in &byzantine_nodes {
            if let NodeStatus::Byzantine(current_strategy) = &state.status[&node] {
                // Adapt based on current conditions
                match current_strategy {
                    ByzantineStrategy::Equivocation => {
                        actions.push(AlpenglowAction::AdaptStrategy {
                            node,
                            new_strategy: ByzantineStrategy::SelectiveEquivocation {
                                min_stake_threshold: 100,
                                target_slots: vec![2, 4],
                            },
                            reason: "Escalating attack".to_string(),
                        });
                    }
                    ByzantineStrategy::WithholdVotes => {
                        actions.push(AlpenglowAction::AdaptStrategy {
                            node,
                            new_strategy: ByzantineStrategy::TimingAttack {
                                delay_votes: true,
                                max_delay: 500,
                                target_path: Some(VotePath::Fast),
                            },
                            reason: "Switching to timing attack".to_string(),
                        });
                    }
                    _ => {}
                }
            }
        }
    }
    
    fn next_state(&self, state: &Self::State, action: Self::Action) -> Option<Self::State> {
        let mut new_state = state.clone();
        
        match action {
            AlpenglowAction::AdvanceTime { delta } => {
                new_state.global_time += delta;
                if new_state.global_time % 10 == 0 && new_state.current_slot < 5 {
                    new_state.current_slot += 1;
                }
            }
            
            AlpenglowAction::Vote { node, slot, block, path } => {
                if matches!(state.status[&node], NodeStatus::Honest) {
                    let stake = *state.stake_distribution.get(&node).unwrap_or(&0);
                    let vote = Vote { node, slot, block, path, stake };
                    
                    if let Some(node_votes) = new_state.votes.get_mut(&node) {
                        if let Some(slot_votes) = node_votes.get_mut(&slot) {
                            // Prevent double voting (honest behavior)
                            if !slot_votes.iter().any(|v| v.block == block && v.path == vote.path) {
                                slot_votes.push(vote);
                            }
                        }
                    }
                }
            }
            
            AlpenglowAction::ByzantineVote { node, strategy, slot } => {
                if let NodeStatus::Byzantine(_) = state.status[&node] {
                    let stake = *state.stake_distribution.get(&node).unwrap_or(&0);
                    self.execute_byzantine_strategy(&mut new_state, node, &strategy, slot, stake);
                }
            }
            
            AlpenglowAction::Certify { slot, path } => {
                let mut all_votes = Vec::new();
                for node_votes in state.votes.values() {
                    if let Some(slot_votes) = node_votes.get(&slot) {
                        for vote in slot_votes {
                            if vote.path == path {
                                all_votes.push(vote.clone());
                            }
                        }
                    }
                }
                
                // Group by block and calculate stake
                let mut block_stakes: HashMap<BlockId, StakeAmount> = HashMap::new();
                let mut block_votes: HashMap<BlockId, HashSet<Vote>> = HashMap::new();
                
                for vote in all_votes {
                    *block_stakes.entry(vote.block).or_insert(0) += vote.stake;
                    block_votes.entry(vote.block).or_insert_with(HashSet::new).insert(vote);
                }
                
                // Check if any block has enough stake for certification
                let required_stake = match path {
                    VotePath::Fast => state.fast_quorum_stake(),
                    VotePath::Slow => state.slow_quorum_stake(),
                };
                
                if let Some((&block, &total_stake)) = block_stakes.iter()
                    .find(|(_, &stake)| stake >= required_stake) {
                    
                    if let Some(votes) = block_votes.get(&block) {
                        let certificate = Certificate {
                            votes: votes.clone(),
                            slot,
                            block,
                            total_stake,
                            path: path.clone(),
                        };
                        new_state.certificates.insert(slot, certificate);
                        
                        // Add to ledger
                        if !new_state.ledger.iter().any(|fb| fb.slot == slot) {
                            new_state.ledger.push(FinalizedBlock {
                                slot,
                                block_id: block,
                                finalization_time: state.global_time,
                                total_stake,
                            });
                        }
                    }
                }
            }
            
            AlpenglowAction::Timeout { node, slot } => {
                if let Some(node_timeouts) = new_state.timeouts.get_mut(&node) {
                    if let Some(timeout_info) = node_timeouts.get_mut(&slot) {
                        timeout_info.count += 1;
                        timeout_info.last_timeout = state.global_time;
                    }
                }
            }
            
            AlpenglowAction::SkipCert { slot } => {
                // Check if enough nodes have timed out
                let timeout_count = state.nodes.iter()
                    .filter(|&&node| {
                        state.timeouts.get(&node)
                            .and_then(|timeouts| timeouts.get(&slot))
                            .map_or(false, |info| info.count >= info.threshold)
                    })
                    .count();
                
                if timeout_count >= (60 * state.nodes.len()) / 100 {
                    let mut timeout_votes = HashSet::new();
                    let mut total_stake = 0;
                    
                    for node_votes in state.votes.values() {
                        if let Some(slot_votes) = node_votes.get(&slot) {
                            for vote in slot_votes {
                                timeout_votes.insert(vote.clone());
                                total_stake += vote.stake;
                            }
                        }
                    }
                    
                    if total_stake >= state.slow_quorum_stake() {
                        let skip_cert = SkipCertificate {
                            slot,
                            timeout_votes,
                            total_stake,
                        };
                        new_state.skip_certs.insert(slot, skip_cert);
                    }
                }
            }
            
            AlpenglowAction::NetworkPartition { nodes_a, nodes_b } => {
                new_state.network_partition = Some(NetworkPartition {
                    partition_a: nodes_a,
                    partition_b: nodes_b,
                    started_at: state.global_time,
                });
            }
            
            AlpenglowAction::HealPartition => {
                new_state.network_partition = None;
            }
            
            AlpenglowAction::FormCoalition { members, strategy } => {
                let total_stake = members.iter()
                    .map(|&node| *state.stake_distribution.get(&node).unwrap_or(&0))
                    .sum();
                
                let coalition = ByzantineCoalition {
                    members: members.clone(),
                    strategy: strategy.clone(),
                    coordination_history: Vec::new(),
                    total_stake,
                    formation_time: state.global_time,
                };
                
                let coalition_index = new_state.byzantine_coalitions.len();
                new_state.byzantine_coalitions.push(coalition);
                new_state.coalition_state.insert(coalition_index, CoalitionState {
                    active: true,
                    current_phase: AttackPhase::Preparation,
                    success_metrics: AttackMetrics {
                        slots_disrupted: 0,
                        certificates_prevented: 0,
                        timeouts_caused: 0,
                        economic_damage: 0,
                    },
                    adaptation_count: 0,
                });
            }
            
            AlpenglowAction::CoordinateAttack { coalition_index, target_slot } => {
                if let Some(coalition_state) = new_state.coalition_state.get_mut(&coalition_index) {
                    coalition_state.current_phase = AttackPhase::Execution;
                    
                    if let Some(coalition) = new_state.byzantine_coalitions.get_mut(coalition_index) {
                        let event = CoordinationEvent {
                            slot: target_slot,
                            event_type: EventType::CoordinatedWithhold { target_slot },
                            participants: coalition.members.clone(),
                            timestamp: state.global_time,
                        };
                        coalition.coordination_history.push(event);
                    }
                }
            }
            
            AlpenglowAction::AdaptStrategy { node, new_strategy, reason: _ } => {
                if let Some(status) = new_state.status.get_mut(&node) {
                    *status = NodeStatus::Byzantine(new_strategy);
                }
            }
            
            AlpenglowAction::TimingManipulation { node: _, delay_ms: _, target_slot: _ } => {
                // Implementation for timing attacks would go here
                // For now, just advance time to simulate delay effects
                new_state.global_time += 10;
            }
        }
        
        Some(new_state)
    }
    
    fn properties(&self) -> Vec<Property<Self>> {
        vec![
            // Enhanced safety property with stake weighting
            Property::always("stake_weighted_safety", |_, state: &Self::State| {
                // No conflicting certificates in same slot
                let mut slot_blocks = HashMap::new();
                for (slot, cert) in &state.certificates {
                    if let Some(existing_block) = slot_blocks.get(slot) {
                        if *existing_block != cert.block {
                            return false; // Conflicting certificates
                        }
                    } else {
                        slot_blocks.insert(*slot, cert.block);
                    }
                }
                true
            }),
            
            // Byzantine resilience
            Property::always("byzantine_resilience", |_, state: &Self::State| {
                let byzantine_stake = state.byzantine_stake();
                let total_stake = state.total_stake();
                
                // Safety should hold if Byzantine stake ≤ 20%
                if byzantine_stake <= (20 * total_stake) / 100 {
                    // All certificates should be consistent
                    state.certificates.values().all(|cert| {
                        let blocks: HashSet<_> = cert.votes.iter().map(|v| v.block).collect();
                        blocks.len() <= 1
                    })
                } else {
                    true // Don't enforce safety if Byzantine threshold exceeded
                }
            }),
            
            // Fast path efficiency
            Property::always("fast_path_efficiency", |_, state: &Self::State| {
                let honest_stake = state.honest_stake();
                let total_stake = state.total_stake();
                
                // If >80% honest stake, fast path should be possible
                if honest_stake >= (80 * total_stake) / 100 {
                    // At least some fast certificates should exist eventually
                    state.certificates.is_empty() || 
                    state.certificates.values().any(|cert| matches!(cert.path, VotePath::Fast))
                } else {
                    true
                }
            }),
            
            // Progress guarantee
            Property::always("progress", |_, state: &Self::State| {
                // For finalized slots, should have certificate or skip certificate
                for slot in 1..state.current_slot {
                    if !state.certificates.contains_key(&slot) && 
                       !state.skip_certs.contains_key(&slot) {
                        return false;
                    }
                }
                true
            }),
            
            // No equivocation by honest nodes
            Property::always("honest_no_equivocation", |_, state: &Self::State| {
                for (&node, node_votes) in &state.votes {
                    if matches!(state.status[&node], NodeStatus::Honest) {
                        for slot_votes in node_votes.values() {
                            // Group votes by slot and path
                            let mut vote_groups: HashMap<(Slot, VotePath), Vec<&Vote>> = HashMap::new();
                            for vote in slot_votes {
                                vote_groups.entry((vote.slot, vote.path.clone()))
                                    .or_insert_with(Vec::new)
                                    .push(vote);
                            }
                            
                            // Each (slot, path) should have at most one block
                            for votes in vote_groups.values() {
                                let blocks: HashSet<_> = votes.iter().map(|v| v.block).collect();
                                if blocks.len() > 1 {
                                    return false; // Equivocation detected
                                }
                            }
                        }
                    }
                }
                true
            }),
        ]
    }
}

impl AlpenglowState {
    fn execute_byzantine_strategy(
        &self,
        state: &mut AlpenglowState,
        node: NodeId,
        strategy: &ByzantineStrategy,
        slot: Slot,
        stake: StakeAmount,
    ) {
        match strategy {
            ByzantineStrategy::Equivocation => {
                // Basic equivocation: vote for multiple blocks
                for block in 0..2 {
                    let vote = Vote { node, slot, block, path: VotePath::Fast, stake };
                    self.add_vote_to_state(state, vote);
                }
            }
            
            ByzantineStrategy::RandomVotes => {
                // Random voting behavior
                let block = (node + slot) % 2;
                let vote = Vote { node, slot, block, path: VotePath::Fast, stake };
                self.add_vote_to_state(state, vote);
            }
            
            ByzantineStrategy::WithholdVotes => {
                // Simply don't vote - no action needed
            }
            
            ByzantineStrategy::SelectiveEquivocation { min_stake_threshold, target_slots } => {
                if stake >= *min_stake_threshold && target_slots.contains(&slot) {
                    // High-impact equivocation on critical slots
                    for block in 0..3 {
                        for path in [VotePath::Fast, VotePath::Slow] {
                            let vote = Vote { node, slot, block, path, stake };
                            self.add_vote_to_state(state, vote);
                        }
                    }
                } else {
                    // Behave honestly to avoid detection
                    let vote = Vote { node, slot, block: 0, path: VotePath::Fast, stake };
                    self.add_vote_to_state(state, vote);
                }
            }
            
            ByzantineStrategy::AdaptiveBehavior { primary_strategy, fallback_strategy, adaptation_threshold } => {
                let timeout_count = state.timeouts.get(&node)
                    .and_then(|timeouts| timeouts.get(&slot))
                    .map(|info| info.count)
                    .unwrap_or(0);
                
                let strategy_to_use = if timeout_count >= *adaptation_threshold {
                    fallback_strategy.as_ref()
                } else {
                    primary_strategy.as_ref()
                };
                
                self.execute_byzantine_strategy(state, node, strategy_to_use, slot, stake);
            }
            
            ByzantineStrategy::CoalitionAttack { coalition_members, attack_type } => {
                self.execute_coalition_attack(state, node, coalition_members, attack_type, slot, stake);
            }
            
            ByzantineStrategy::TimingAttack { delay_votes, max_delay, target_path } => {
                if *delay_votes {
                    // Simulate delay by advancing time
                    state.global_time += std::cmp::min(*max_delay, 1000);
                }
                
                let path = target_path.clone().unwrap_or(VotePath::Slow);
                let vote = Vote { node, slot, block: 0, path, stake };
                self.add_vote_to_state(state, vote);
            }
            
            ByzantineStrategy::StakeBasedAttack { reserve_stake_for_critical_slots, activation_threshold, min_profit_margin: _ } => {
                if stake >= *activation_threshold {
                    if *reserve_stake_for_critical_slots && slot % 3 == 0 {
                        // Critical slot: maximize disruption
                        for block in 0..3 {
                            let vote = Vote { node, slot, block, path: VotePath::Fast, stake };
                            self.add_vote_to_state(state, vote);
                        }
                    } else {
                        // Regular slot: conservative attack
                        let vote = Vote { node, slot, block: 1, path: VotePath::Fast, stake };
                        self.add_vote_to_state(state, vote);
                    }
                }
            }
        }
    }
    
    fn add_vote_to_state(&self, state: &mut AlpenglowState, vote: Vote) {
        if let Some(node_votes) = state.votes.get_mut(&vote.node) {
            if let Some(slot_votes) = node_votes.get_mut(&vote.slot) {
                slot_votes.push(vote);
            }
        }
    }
    
    fn execute_coalition_attack(
        &self,
        state: &mut AlpenglowState,
        node: NodeId,
        coalition_members: &[NodeId],
        attack_type: &CoalitionAttackType,
        slot: Slot,
        stake: StakeAmount,
    ) {
        if !coalition_members.contains(&node) {
            return;
        }
        
        match attack_type {
            CoalitionAttackType::SplitVote { target_blocks } => {
                let node_index = coalition_members.iter().position(|&n| n == node).unwrap_or(0);
                let block = target_blocks.get(node_index % target_blocks.len()).copied().unwrap_or(0);
                let vote = Vote { node, slot, block, path: VotePath::Fast, stake };
                self.add_vote_to_state(state, vote);
            }
            
            CoalitionAttackType::DelayedFlood { delay_until_slot } => {
                if slot >= *delay_until_slot {
                    // Flood with multiple votes
                    for block in 0..2 {
                        for path in [VotePath::Fast, VotePath::Slow] {
                            let vote = Vote { node, slot, block, path, stake };
                            self.add_vote_to_state(state, vote);
                        }
                    }
                }
            }
            
            CoalitionAttackType::StrategicTargeting { high_priority_slots, disruption_threshold: _ } => {
                if high_priority_slots.contains(&slot) {
                    // Maximum disruption on targeted slots
                    for block in 0..3 {
                        let vote = Vote { node, slot, block, path: VotePath::Fast, stake };
                        self.add_vote_to_state(state, vote);
                    }
                } else {
                    // Normal behavior to avoid detection
                    let vote = Vote { node, slot, block: 0, path: VotePath::Fast, stake };
                    self.add_vote_to_state(state, vote);
                }
            }
            
            CoalitionAttackType::CertificateManipulation { target_path, manipulation_type } => {
                match manipulation_type {
                    CertManipulationType::PreventCertification => {
                        // Withhold votes to prevent certificate formation
                        // No vote is cast
                    }
                    
                    CertManipulationType::ConflictingCertificates => {
                        // Vote for different blocks to create conflicts
                        let block = (node + slot) % 3;
                        let vote = Vote { node, slot, block, path: target_path.clone(), stake };
                        self.add_vote_to_state(state, vote);
                    }
                    
                    CertManipulationType::DelayedCertification { delay_slots: _ } => {
                        // Delay voting by a few slots
                        if slot > 2 {
                            let delayed_slot = slot - 2;
                            let vote = Vote { node, slot: delayed_slot, block: 0, path: target_path.clone(), stake };
                            self.add_vote_to_state(state, vote);
                        }
                    }
                }
            }
        }
    }
}