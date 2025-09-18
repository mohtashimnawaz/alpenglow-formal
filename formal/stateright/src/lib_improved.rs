use stateright::*;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use serde::{Deserialize, Serialize};

/// Alpenglow Consensus Protocol - Enhanced Stateright Model
/// Implements Votor dual-path consensus with improved stake weighting and Byzantine behaviors

pub mod scalability;
pub use scalability::*;

pub type NodeId = u32;
pub type Slot = u32;
pub type BlockId = u32;
pub type StakeAmount = u64;
pub type Timestamp = u64;
pub type Round = u32;

// Additional structures needed for statistical model checking
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash)]
pub struct Node {
    pub id: NodeId,
    pub stake: StakeAmount,
    pub is_byzantine: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash)]
pub struct Transaction {
    pub id: u64,
    pub sender: NodeId,
    pub recipient: NodeId,
    pub amount: u64,
    pub timestamp: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash)]
pub struct PartialBlock {
    pub height: u32,
    pub parent_hash: u64,
    pub transactions: Vec<Transaction>,
    pub proposer: NodeId,
    pub timestamp: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Message {
    Heartbeat,
    Proposal(PartialBlock),
    Vote(Vote),
    Certificate(Certificate),
}

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
    pub network_state: NetworkSimulationState,
    pub message_queue: MessageQueue,
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

/// Network simulation structures for realistic modeling
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NetworkSimulationState {
    pub latency_model: LatencyModel,
    pub packet_loss_rate: f64, // 0.0 to 1.0
    pub bandwidth_limits: HashMap<(NodeId, NodeId), Bandwidth>,
    pub congestion_state: CongestionState,
    pub failure_injections: Vec<NetworkFailure>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MessageQueue {
    pub pending_messages: Vec<PendingMessage>,
    pub delivered_messages: Vec<DeliveredMessage>,
    pub message_counter: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PendingMessage {
    pub id: u64,
    pub from: NodeId,
    pub to: NodeId,
    pub content: MessageContent,
    pub send_time: Timestamp,
    pub scheduled_delivery_time: Timestamp,
    pub priority: MessagePriority,
    pub retry_count: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DeliveredMessage {
    pub id: u64,
    pub from: NodeId,
    pub to: NodeId,
    pub content: MessageContent,
    pub send_time: Timestamp,
    pub delivery_time: Timestamp,
    pub actual_latency: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MessageContent {
    Vote(Vote),
    Certificate(Certificate),
    SkipCertificate(SkipCertificate),
    Gossip { data: Vec<u8> },
    Heartbeat { sequence: u64 },
    CoalitionCoordination { coalition_id: usize, instruction: CoordinationInstruction },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CoordinationInstruction {
    PrepareAttack { target_slot: Slot },
    ExecuteAttack { strategy: CoalitionAttackType },
    AbortAttack { reason: String },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MessagePriority {
    Critical, // Votes, certificates
    High,     // Skip certificates, timeouts
    Normal,   // Gossip, heartbeats
    Low,      // Coalition coordination
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash)]
pub enum LatencyModel {
    Constant { latency_ms: u64 },
    Uniform { min_ms: u64, max_ms: u64 },
    Normal { mean_ms: u64, std_dev_ms: u64 }, // Changed to u64 for Hash
    Realistic { 
        base_latency_ms: u64,
        distance_factor: u64, // Changed to u64 for Hash
        congestion_multiplier: u64, // Changed to u64 for Hash
    },
}

pub type Bandwidth = u64; // bytes per second

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CongestionState {
    pub current_utilization: HashMap<(NodeId, NodeId), f64>, // 0.0 to 1.0+
    pub congestion_threshold: f64, // when to start dropping/delaying
    pub recovery_rate: f64, // how fast congestion clears
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NetworkFailure {
    pub failure_type: FailureType,
    pub start_time: Timestamp,
    pub duration: Timestamp,
    pub affected_nodes: Vec<NodeId>,
    pub severity: f64, // 0.0 to 1.0
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum FailureType {
    LinkFailure { from: NodeId, to: NodeId },
    NodeIsolation { node: NodeId },
    PartialPartition { partition_a: Vec<NodeId>, partition_b: Vec<NodeId> },
    PacketLoss { loss_rate: f64 },
    LatencySpike { multiplier: f64 },
    BandwidthReduction { factor: f64 },
}

impl Default for NetworkSimulationState {
    fn default() -> Self {
        Self {
            latency_model: LatencyModel::Constant { latency_ms: 50 },
            packet_loss_rate: 0.01, // 1% packet loss
            bandwidth_limits: HashMap::new(),
            congestion_state: CongestionState::default(),
            failure_injections: Vec::new(),
        }
    }
}

impl Default for MessageQueue {
    fn default() -> Self {
        Self {
            pending_messages: Vec::new(),
            delivered_messages: Vec::new(),
            message_counter: 0,
        }
    }
}

impl Default for CongestionState {
    fn default() -> Self {
        Self {
            current_utilization: HashMap::new(),
            congestion_threshold: 0.8, // 80% utilization threshold
            recovery_rate: 0.1, // 10% recovery per time unit
        }
    }
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
    
    // Network simulation actions
    SendMessage { from: NodeId, to: NodeId, content: MessageContent, priority: MessagePriority },
    DeliverMessage { message_id: u64 },
    DropMessage { message_id: u64, reason: String },
    InjectNetworkFailure { failure: NetworkFailure },
    RecoverFromFailure { failure_index: usize },
    UpdateLatencyModel { new_model: LatencyModel },
    AdjustBandwidth { from: NodeId, to: NodeId, new_bandwidth: Bandwidth },
    SimulateCongestion { links: Vec<(NodeId, NodeId)>, intensity: f64 },
}

#[derive(Clone, Debug, PartialEq)]
pub enum NetworkAction {
    SendMessage { from: NodeId, to: NodeId, message: Message, priority: MessagePriority },
    DeliverMessage { message_id: u64 },
    DropMessage { message_id: u64, reason: String },
    InjectNetworkFailure { failure: NetworkFailure },
    RecoverFromFailure { failure_index: usize },
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
            network_state: NetworkSimulationState::default(),
            message_queue: MessageQueue::default(),
        }
    }
    
    // Alternative constructor that takes Node structs for statistical testing
    pub fn new_with_nodes(nodes: Vec<Node>, stake_map: HashMap<NodeId, StakeAmount>) -> Self {
        let node_ids: Vec<NodeId> = nodes.iter().map(|n| n.id).collect();
        let mut stake_distribution = HashMap::new();
        let mut byzantine_coalitions = Vec::new();
        let mut byzantine_members = Vec::new();
        
        // Extract stake and Byzantine info from Node structs
        for node in &nodes {
            stake_distribution.insert(node.id, node.stake);
            if node.is_byzantine {
                byzantine_members.push(node.id);
            }
        }
        
        // Create Byzantine coalition if we have Byzantine nodes
        if !byzantine_members.is_empty() {
            let total_stake: u64 = byzantine_members.iter().map(|&id| stake_map[&id]).sum();
            byzantine_coalitions.push(ByzantineCoalition {
                members: byzantine_members,
                strategy: CoalitionAttackType::SplitVote { 
                    target_blocks: vec![]
                },
                coordination_history: Vec::new(),
                total_stake,
                formation_time: 0,
            });
        }
        
        let mut state = Self::new(node_ids, stake_distribution);
        state.byzantine_coalitions = byzantine_coalitions;
        
        // Update status for Byzantine nodes
        for node in &nodes {
            if node.is_byzantine {
                state.status.insert(node.id, NodeStatus::Byzantine(ByzantineStrategy::Equivocation));
            }
        }
        
        state
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
        
        // Network simulation actions
        self.generate_network_actions(state, actions);
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
            
            // Network simulation action handlers
            AlpenglowAction::SendMessage { from, to, content, priority } => {
                self.handle_send_message(&mut new_state, from, to, content, priority);
            }
            
            AlpenglowAction::DeliverMessage { message_id } => {
                self.handle_deliver_message(&mut new_state, message_id);
            }
            
            AlpenglowAction::DropMessage { message_id, reason: _ } => {
                new_state.message_queue.pending_messages.retain(|msg| msg.id != message_id);
            }
            
            AlpenglowAction::InjectNetworkFailure { failure } => {
                new_state.network_state.failure_injections.push(failure);
            }
            
            AlpenglowAction::RecoverFromFailure { failure_index } => {
                if failure_index < new_state.network_state.failure_injections.len() {
                    new_state.network_state.failure_injections.remove(failure_index);
                }
            }
            
            AlpenglowAction::UpdateLatencyModel { new_model } => {
                new_state.network_state.latency_model = new_model;
            }
            
            AlpenglowAction::AdjustBandwidth { from, to, new_bandwidth } => {
                new_state.network_state.bandwidth_limits.insert((from, to), new_bandwidth);
            }
            
            AlpenglowAction::SimulateCongestion { links, intensity } => {
                for (from, to) in links {
                    new_state.network_state.congestion_state.current_utilization.insert((from, to), intensity);
                }
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
    
    /// Network simulation helper methods
    fn generate_network_actions(&self, state: &AlpenglowState, actions: &mut Vec<AlpenglowAction>) {
        // Message delivery actions
        for pending_msg in &state.message_queue.pending_messages {
            if pending_msg.scheduled_delivery_time <= state.global_time {
                actions.push(AlpenglowAction::DeliverMessage { 
                    message_id: pending_msg.id 
                });
            }
        }
        
        // Spontaneous message sending (gossip, heartbeats)
        for &from in &state.nodes {
            if matches!(state.status[&from], NodeStatus::Honest | NodeStatus::Byzantine(_)) {
                for &to in &state.nodes {
                    if from != to {
                        // Heartbeat messages
                        actions.push(AlpenglowAction::SendMessage {
                            from,
                            to,
                            content: MessageContent::Heartbeat { sequence: state.global_time },
                            priority: MessagePriority::Normal,
                        });
                        
                        // Vote propagation (if node has votes)
                        if let Some(node_votes) = state.votes.get(&from) {
                            for slot_votes in node_votes.values() {
                                for vote in slot_votes {
                                    actions.push(AlpenglowAction::SendMessage {
                                        from,
                                        to,
                                        content: MessageContent::Vote(vote.clone()),
                                        priority: MessagePriority::Critical,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Network failure injection
        if state.network_state.failure_injections.is_empty() && state.global_time > 10 {
            // Inject random network failures
            actions.push(AlpenglowAction::InjectNetworkFailure {
                failure: NetworkFailure {
                    failure_type: FailureType::PacketLoss { loss_rate: 0.1 },
                    start_time: state.global_time,
                    duration: 20,
                    affected_nodes: state.nodes[0..state.nodes.len()/2].to_vec(),
                    severity: 0.3,
                },
            });
            
            actions.push(AlpenglowAction::InjectNetworkFailure {
                failure: NetworkFailure {
                    failure_type: FailureType::LatencySpike { multiplier: 3.0 },
                    start_time: state.global_time,
                    duration: 15,
                    affected_nodes: state.nodes.clone(),
                    severity: 0.5,
                },
            });
        }
        
        // Congestion simulation
        if state.global_time % 20 == 0 {
            let mut congested_links = Vec::new();
            for &from in &state.nodes {
                for &to in &state.nodes {
                    if from != to && (from + to) % 3 == 0 {
                        congested_links.push((from, to));
                    }
                }
            }
            if !congested_links.is_empty() {
                actions.push(AlpenglowAction::SimulateCongestion {
                    links: congested_links,
                    intensity: 0.9,
                });
            }
        }
        
        // Dynamic latency model updates
        if state.global_time % 50 == 0 {
            actions.push(AlpenglowAction::UpdateLatencyModel {
                new_model: LatencyModel::Normal { 
                    mean_ms: 100, 
                    std_dev_ms: 25 
                },
            });
        }
    }
    
    fn handle_send_message(
        &self,
        state: &mut AlpenglowState,
        from: NodeId,
        to: NodeId,
        content: MessageContent,
        priority: MessagePriority,
    ) {
        // Check if nodes are in same partition
        if let Some(partition) = &state.network_partition {
            let from_in_a = partition.partition_a.contains(&from);
            let to_in_a = partition.partition_a.contains(&to);
            if from_in_a != to_in_a {
                // Cross-partition message - drop it
                return;
            }
        }
        
        // Check for active network failures
        let mut should_drop = false;
        let mut latency_multiplier = 1.0;
        
        for failure in &state.network_state.failure_injections {
            if failure.start_time <= state.global_time && 
               state.global_time < failure.start_time + failure.duration {
                
                match &failure.failure_type {
                    FailureType::LinkFailure { from: f, to: t } => {
                        if (*f == from && *t == to) || (*f == to && *t == from) {
                            should_drop = true;
                        }
                    }
                    FailureType::NodeIsolation { node } => {
                        if *node == from || *node == to {
                            should_drop = true;
                        }
                    }
                    FailureType::PacketLoss { loss_rate } => {
                        // Simple hash-based deterministic "randomness"
                        let hash_val = (from + to + (state.global_time as u32)) % 100;
                        if (hash_val as f64) / 100.0 < *loss_rate {
                            should_drop = true;
                        }
                    }
                    FailureType::LatencySpike { multiplier } => {
                        latency_multiplier *= multiplier;
                    }
                    _ => {}
                }
            }
        }
        
        if should_drop {
            return;
        }
        
        // Calculate delivery time based on latency model
        let base_latency = self.calculate_latency(state, from, to);
        let final_latency = (base_latency as f64 * latency_multiplier) as u64;
        
        // Apply congestion effects
        let congestion_factor = state.network_state.congestion_state
            .current_utilization
            .get(&(from, to))
            .copied()
            .unwrap_or(0.0);
        
        let congestion_delay = if congestion_factor > state.network_state.congestion_state.congestion_threshold {
            (final_latency as f64 * congestion_factor) as u64
        } else {
            0
        };
        
        let total_latency = final_latency + congestion_delay;
        
        // Create pending message
        let message_id = state.message_queue.message_counter;
        state.message_queue.message_counter += 1;
        
        let pending_message = PendingMessage {
            id: message_id,
            from,
            to,
            content,
            send_time: state.global_time,
            scheduled_delivery_time: state.global_time + total_latency,
            priority,
            retry_count: 0,
        };
        
        state.message_queue.pending_messages.push(pending_message);
    }
    
    fn handle_deliver_message(&self, state: &mut AlpenglowState, message_id: u64) {
        if let Some(pos) = state.message_queue.pending_messages.iter().position(|msg| msg.id == message_id) {
            let message = state.message_queue.pending_messages.remove(pos);
            let content_clone = message.content.clone();
            
            // Process the message content
            match &message.content {
                MessageContent::Vote(vote) => {
                    // Deliver vote to receiving node
                    if let Some(node_votes) = state.votes.get_mut(&message.to) {
                        if let Some(slot_votes) = node_votes.get_mut(&vote.slot) {
                            // Add vote if not already present (avoid duplicates)
                            if !slot_votes.iter().any(|v| v.node == vote.node && v.block == vote.block && v.path == vote.path) {
                                slot_votes.push(vote.clone());
                            }
                        }
                    }
                }
                MessageContent::Certificate(cert) => {
                    // Deliver certificate
                    state.certificates.insert(cert.slot, cert.clone());
                }
                MessageContent::SkipCertificate(skip_cert) => {
                    state.skip_certs.insert(skip_cert.slot, skip_cert.clone());
                }
                MessageContent::CoalitionCoordination { coalition_id, instruction } => {
                    // Handle coalition coordination
                    if let Some(coalition_state) = state.coalition_state.get_mut(coalition_id) {
                        match instruction {
                            CoordinationInstruction::PrepareAttack { target_slot: _ } => {
                                coalition_state.current_phase = AttackPhase::Preparation;
                            }
                            CoordinationInstruction::ExecuteAttack { strategy: _ } => {
                                coalition_state.current_phase = AttackPhase::Execution;
                            }
                            CoordinationInstruction::AbortAttack { reason: _ } => {
                                coalition_state.active = false;
                            }
                        }
                    }
                }
                _ => {} // Heartbeat, gossip - just update delivery metrics
            }
            
            // Record successful delivery
            let delivered_message = DeliveredMessage {
                id: message.id,
                from: message.from,
                to: message.to,
                content: content_clone,
                send_time: message.send_time,
                delivery_time: state.global_time,
                actual_latency: state.global_time - message.send_time,
            };
            
            state.message_queue.delivered_messages.push(delivered_message);
        }
    }
    
    pub fn calculate_latency(&self, state: &AlpenglowState, from: NodeId, to: NodeId) -> u64 {
        match &state.network_state.latency_model {
            LatencyModel::Constant { latency_ms } => *latency_ms,
            LatencyModel::Uniform { min_ms, max_ms } => {
                // Simple hash-based deterministic "randomness"
                let range = max_ms - min_ms;
                if range == 0 { return *min_ms; }
                let hash_val = (from + to + (state.global_time as u32)) % (range as u32);
                min_ms + (hash_val as u64)
            }
            LatencyModel::Normal { mean_ms, std_dev_ms } => {
                // Simplified normal distribution using hash
                let hash_val = (from * 17 + to * 31 + (state.global_time * 7) as u32) % 1000;
                let normalized = (hash_val as f64) / 1000.0; // 0.0 to 1.0
                let z_score = (normalized - 0.5) * 4.0; // Rough normal distribution
                let latency = (*mean_ms as f64) + (z_score * (*std_dev_ms as f64));
                latency.max(1.0) as u64
            }
            LatencyModel::Realistic { base_latency_ms, distance_factor, congestion_multiplier } => {
                let distance = ((from as i32 - to as i32).abs()) as f64;
                let distance_latency = distance * (*distance_factor as f64);
                let congestion = state.network_state.congestion_state
                    .current_utilization
                    .get(&(from, to))
                    .copied()
                    .unwrap_or(0.0);
                let congestion_latency = congestion * (*congestion_multiplier as f64);
                
                ((*base_latency_ms as f64) + distance_latency + congestion_latency) as u64
            }
        }
    }
}

// Statistical Model Checking for Large Node Sets (100+ nodes)
#[derive(Debug, Clone, PartialEq)]
pub struct StatisticalConfig {
    pub max_samples: u32,
    pub confidence_level: f64,  // e.g., 0.95 for 95%
    pub error_bound: f64,       // Maximum acceptable error
    pub parallel_workers: usize,
    pub max_depth: Option<u32>, // Limit exploration depth
}

impl Default for StatisticalConfig {
    fn default() -> Self {
        Self {
            max_samples: 10000,
            confidence_level: 0.95,
            error_bound: 0.05,
            parallel_workers: 4,
            max_depth: Some(100),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StatisticalResult {
    pub samples_taken: u32,
    pub property_satisfied_count: u32,
    pub estimated_probability: f64,
    pub confidence_interval: (f64, f64),
    pub convergence_achieved: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SamplingStrategy {
    pub sampling_type: SamplingType,
    pub priority_weights: std::collections::HashMap<String, f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SamplingType {
    UniformRandom,
    ImportanceSampling,
    StratifiedSampling,
    AdaptiveSampling,
}

impl Default for SamplingStrategy {
    fn default() -> Self {
        Self {
            sampling_type: SamplingType::UniformRandom,
            priority_weights: std::collections::HashMap::new(),
        }
    }
}

// State space optimization for large networks
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompactState {
    pub consensus_hash: u64,
    pub network_hash: u64,
    pub byzantine_hash: u64,
    pub essential_metrics: EssentialMetrics,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EssentialMetrics {
    pub committed_blocks: u32,
    pub active_byzantine_nodes: u32,
    pub network_partitions: u32,
    pub average_latency: u64,
}

impl AlpenglowState {
    // Create a compact representation for large-scale model checking
    pub fn to_compact_state(&self) -> CompactState {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut consensus_hasher = DefaultHasher::new();
        self.current_slot.hash(&mut consensus_hasher);
        self.ledger.len().hash(&mut consensus_hasher);
        
        let mut network_hasher = DefaultHasher::new();
        self.network_state.latency_model.hash(&mut network_hasher);
        self.message_queue.pending_messages.len().hash(&mut network_hasher);
        
        let mut byzantine_hasher = DefaultHasher::new();
        for coalition in &self.byzantine_coalitions {
            coalition.members.hash(&mut byzantine_hasher);
            coalition.total_stake.hash(&mut byzantine_hasher);
        }
        
        CompactState {
            consensus_hash: consensus_hasher.finish(),
            network_hash: network_hasher.finish(),
            byzantine_hash: byzantine_hasher.finish(),
            essential_metrics: EssentialMetrics {
                committed_blocks: self.ledger.len() as u32,
                active_byzantine_nodes: self.byzantine_coalitions
                    .iter()
                    .map(|c| c.members.len() as u32)
                    .sum(),
                network_partitions: self.network_state.failure_injections.len() as u32,
                average_latency: self.message_queue.delivered_messages
                    .iter()
                    .map(|m| m.actual_latency)
                    .sum::<u64>()
                    .checked_div(self.message_queue.delivered_messages.len() as u64)
                    .unwrap_or(0),
            },
        }
    }
    
    // Simplified statistical properties checking for scalability demo
    pub fn verify_scalability_properties(&self) -> bool {
        // Basic scalability properties that should hold for large networks
        let total_nodes = self.nodes.len();
        let byzantine_nodes = self.byzantine_coalitions
            .iter()
            .map(|c| c.members.len())
            .sum::<usize>();
        let honest_nodes = total_nodes - byzantine_nodes;
        
        // Property 1: More than 2/3 honest nodes (Byzantine fault tolerance)
        let byzantine_resilient = honest_nodes > byzantine_nodes * 2;
        
        // Property 2: Total stake is correctly distributed
        let expected_stake = self.total_stake();
        let actual_stake: u64 = self.stake_distribution.values().sum();
        let stake_consistent = expected_stake == actual_stake;
        
        // Property 3: Network state is properly initialized
        let network_initialized = !self.nodes.is_empty() && 
                                   !self.stake_distribution.is_empty() &&
                                   self.stake_distribution.len() == total_nodes;
        
        byzantine_resilient && stake_consistent && network_initialized
    }
    
}
