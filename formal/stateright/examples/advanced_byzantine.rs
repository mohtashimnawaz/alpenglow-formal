// Example: Advanced Byzantine Behaviors in Alpenglow
// This example demonstrates sophisticated Byzantine attacks including coordinated coalitions,
// network manipulation, and economic incentive-based attacks.

use alpenglow_stateright::{*, lib_advanced::*, model_advanced::*};
use std::collections::HashMap;
use stateright::*;

/// Demonstrates coordinated Byzantine coalition attack
fn coalition_attack_example() {
    println!("=== Byzantine Coalition Attack Example ===");
    
    let nodes = vec![1, 2, 3, 4, 5, 6, 7];
    let stake_distribution: HashMap<NodeId, StakeAmount> = nodes.iter()
        .enumerate()
        .map(|(i, &node)| {
            let stake = if i < 2 { 150_000 } else { 100_000 }; // Give attackers more stake
            (node, stake)
        })
        .collect();
    
    let model = AlpenglowAdvancedModel::new(
        nodes.clone(),
        stake_distribution,
        NetworkConditions {
            base_latency: 100,
            jitter: 50,
            packet_loss_rate: 0.02,
            corruption_rate: 0.01,
        },
        EconomicParams {
            base_reward: 1000,
            penalty_multiplier: 3.0,
            reward_pool: 500_000,
            inflation_rate: 0.03,
            validator_reward_share: 0.7,
        },
    );
    
    let mut state = model.init_states()[0].clone();
    
    // Form a Byzantine coalition
    state.status.insert(1, NodeStatus::Byzantine(ByzantineStrategy::Equivocation));
    state.status.insert(2, NodeStatus::Byzantine(ByzantineStrategy::SelectiveEquivocation {
        target_nodes: vec![3, 4, 5].into_iter().collect(),
        probability: 0.8,
    }));
    
    println!("Initial state:");
    println!("  Total nodes: {}", state.nodes.len());
    println!("  Byzantine nodes: {}", state.nodes.iter().filter(|&&n| matches!(state.status[&n], NodeStatus::Byzantine(_))).count());
    println!("  Current slot: {}", state.current_slot);
    
    // Generate and apply coalition formation action
    let mut actions = Vec::new();
    model.actions(&state, &mut actions);
    
    if let Some(coalition_action) = actions.iter().find(|action| {
        matches!(action, AlpenglowAdvancedAction::FormCoalition { .. })
    }) {
        println!("  Forming Byzantine coalition...");
        state = model.next_state(&state, coalition_action.clone()).unwrap();
        println!("  Coalition formed with {} members", state.byzantine_coalitions[0].members.len());
    }
    
    // Execute coordinated attack
    for round in 1..=3 {
        println!("\n--- Attack Round {} ---", round);
        
        let mut round_actions = Vec::new();
        model.actions(&state, &mut round_actions);
        
        // Look for coordinated attack actions
        if let Some(attack_action) = round_actions.iter().find(|action| {
            matches!(action, AlpenglowAdvancedAction::CoordinateAttack { .. })
        }) {
            println!("  Executing coordinated attack on slot {}", state.current_slot);
            state = model.next_state(&state, attack_action.clone()).unwrap();
            
            // Check for detected violations
            let violations: Vec<_> = state.nodes.iter()
                .flat_map(|&node| state.detect_byzantine_behavior(node, state.current_slot))
                .collect();
            
            println!("  Detected {} Byzantine violations", violations.len());
            
            // Check economic penalties
            let total_penalties: StakeAmount = state.economic_state.penalties.values().sum();
            println!("  Total penalties applied: {}", total_penalties);
        }
        
        // Advance time
        state = model.next_state(&state, AlpenglowAdvancedAction::AdvanceTime { delta: 500 }).unwrap();
    }
    
    println!("\n=== Attack Results ===");
    println!("Final slot: {}", state.current_slot);
    println!("Certificates created: {}", state.certificates.len());
    println!("Skip certificates: {}", state.skip_certs.len());
    println!("Safety violated: {}", state.is_safety_violated());
    println!("Liveness violated: {}", state.is_liveness_violated(3));
}

/// Demonstrates network partition attack
fn network_partition_example() {
    println!("\n=== Network Partition Attack Example ===");
    
    let nodes = vec![1, 2, 3, 4, 5, 6];
    let stake_distribution: HashMap<NodeId, StakeAmount> = nodes.iter()
        .map(|&node| (node, 100_000))
        .collect();
    
    let model = AlpenglowAdvancedModel::new(
        nodes.clone(),
        stake_distribution,
        NetworkConditions {
            base_latency: 200,
            jitter: 100,
            packet_loss_rate: 0.15, // High packet loss
            corruption_rate: 0.05,  // High corruption
        },
        EconomicParams::default(),
    );
    
    let mut state = model.init_states()[0].clone();
    
    // Create network partition
    let partition_a: std::collections::HashSet<NodeId> = vec![1, 2, 3].into_iter().collect();
    let partition_b: std::collections::HashSet<NodeId> = vec![4, 5, 6].into_iter().collect();
    
    let partition_action = AlpenglowAdvancedAction::CreatePartition {
        partition_type: PartitionType::Complete,
        nodes: vec![partition_a.clone(), partition_b.clone()],
    };
    
    println!("Creating network partition:");
    println!("  Partition A: {:?}", partition_a);
    println!("  Partition B: {:?}", partition_b);
    
    state = model.next_state(&state, partition_action).unwrap();
    
    // Test communication
    let can_communicate_within = state.can_nodes_communicate(1, 2);
    let can_communicate_across = state.can_nodes_communicate(1, 4);
    
    println!("  Communication within partition: {}", can_communicate_within);
    println!("  Communication across partition: {}", can_communicate_across);
    
    // Run consensus with partition
    for round in 1..=5 {
        let mut actions = Vec::new();
        model.actions(&state, &mut actions);
        
        // Apply first available action
        if let Some(action) = actions.first() {
            state = model.next_state(&state, action.clone()).unwrap();
        }
        
        // Advance time
        state = model.next_state(&state, AlpenglowAdvancedAction::AdvanceTime { delta: 300 }).unwrap();
    }
    
    println!("Partition results:");
    println!("  Certificates: {}", state.certificates.len());
    println!("  Skip certs: {}", state.skip_certs.len());
    println!("  Current slot: {}", state.current_slot);
}

/// Demonstrates economic attack scenarios
fn economic_attack_example() {
    println!("\n=== Economic Attack Example ===");
    
    let nodes = vec![1, 2, 3, 4, 5];
    let stake_distribution: HashMap<NodeId, StakeAmount> = vec![
        (1, 200_000), // High stake attacker
        (2, 100_000),
        (3, 100_000),
        (4, 100_000),
        (5, 100_000),
    ].into_iter().collect();
    
    let model = AlpenglowAdvancedModel::new(
        nodes.clone(),
        stake_distribution,
        NetworkConditions::default(),
        EconomicParams {
            base_reward: 2000,
            penalty_multiplier: 2.5,
            reward_pool: 1_000_000,
            inflation_rate: 0.05,
            validator_reward_share: 0.8,
        },
    );
    
    let mut state = model.init_states()[0].clone();
    
    // Set up economically motivated attacker
    state.status.insert(1, NodeStatus::Byzantine(ByzantineStrategy::StakeBasedAttack {
        target_stake_threshold: 150_000,
        attack_probability: 0.9,
    }));
    
    // Set up rational validator
    state.status.insert(2, NodeStatus::Economically(EconomicallyMotivated::RationalValidator {
        profit_threshold: 1500.0,
    }));
    
    println!("Economic setup:");
    println!("  Reward pool: {}", state.economic_state.reward_pool);
    println!("  High stake attacker (node 1): {} stake", state.effective_stake(1));
    println!("  Rational validator (node 2): {} stake", state.effective_stake(2));
    
    // Simulate economic attacks
    for round in 1..=4 {
        println!("\n--- Economic Round {} ---", round);
        
        let mut actions = Vec::new();
        model.actions(&state, &mut actions);
        
        // Check economic incentives for different actions
        for &node in &[1, 2] {
            let vote_incentive = state.calculate_economic_incentive(
                node,
                &AlpenglowAdvancedAction::Vote {
                    node,
                    slot: state.current_slot,
                    block: state.current_slot,
                    path: VotePath::Fast,
                }
            );
            
            println!("  Node {} vote incentive: {}", node, vote_incentive);
        }
        
        // Apply reward distribution
        if let Some(reward_action) = actions.iter().find(|action| {
            matches!(action, AlpenglowAdvancedAction::DistributeRewards { .. })
        }) {
            state = model.next_state(&state, reward_action.clone()).unwrap();
        }
        
        // Apply slashing if violations detected
        for slashing_action in actions.iter().filter(|action| {
            matches!(action, AlpenglowAdvancedAction::ApplySlashing { .. })
        }).take(2) {
            state = model.next_state(&state, slashing_action.clone()).unwrap();
        }
        
        // Advance time
        state = model.next_state(&state, AlpenglowAdvancedAction::AdvanceTime { delta: 400 }).unwrap();
    }
    
    println!("\nEconomic results:");
    for &node in &nodes {
        let rewards = state.economic_state.rewards.get(&node).unwrap_or(&0);
        let penalties = state.economic_state.penalties.get(&node).unwrap_or(&0);
        let net_gain = (*rewards as i64) - (*penalties as i64);
        println!("  Node {}: rewards={}, penalties={}, net={}", node, rewards, penalties, net_gain);
    }
    
    let total_slashing: StakeAmount = state.economic_state.slashing_conditions.iter()
        .map(|slash| slash.penalty_amount)
        .sum();
    println!("  Total slashing applied: {}", total_slashing);
}

/// Demonstrates adaptive Byzantine strategy
fn adaptive_strategy_example() {
    println!("\n=== Adaptive Byzantine Strategy Example ===");
    
    let nodes = vec![1, 2, 3, 4, 5];
    let stake_distribution: HashMap<NodeId, StakeAmount> = nodes.iter()
        .map(|&node| (node, 100_000))
        .collect();
    
    let model = AlpenglowAdvancedModel::new(
        nodes.clone(),
        stake_distribution,
        NetworkConditions::default(),
        EconomicParams::default(),
    );
    
    let mut state = model.init_states()[0].clone();
    
    // Set up adaptive Byzantine node
    state.status.insert(1, NodeStatus::Byzantine(ByzantineStrategy::AdaptiveStrategy {
        success_threshold: 0.6,
        adaptation_rate: 0.2,
        current_strategy: Box::new(ByzantineStrategy::Equivocation),
    }));
    
    println!("Adaptive strategy simulation:");
    
    // Simulate multiple rounds with adaptation
    for round in 1..=6 {
        println!("\n--- Adaptive Round {} ---", round);
        
        let mut actions = Vec::new();
        model.actions(&state, &mut actions);
        
        // Execute Byzantine votes
        for action in actions.iter().filter(|action| {
            matches!(action, AlpenglowAdvancedAction::ByzantineVote { node: 1, .. })
        }).take(1) {
            state = model.next_state(&state, action.clone()).unwrap();
        }
        
        // Check detection
        let violations = state.detect_byzantine_behavior(1, state.current_slot);
        let detected = !violations.is_empty();
        
        println!("  Byzantine behavior detected: {}", detected);
        
        if detected {
            println!("  Violations: {:?}", violations.iter().map(|v| format!("{:?}", v)).collect::<Vec<_>>());
            
            // Apply penalties
            for violation in violations {
                let penalty = state.effective_stake(1) / 10;
                let slashing_action = AlpenglowAdvancedAction::ApplySlashing {
                    node: 1,
                    violation,
                    penalty,
                };
                state = model.next_state(&state, slashing_action).unwrap();
            }
        }
        
        // The adaptive strategy should change based on success/failure
        // (In a full implementation, this would be handled automatically)
        
        // Advance time and slot
        state = model.next_state(&state, AlpenglowAdvancedAction::AdvanceTime { delta: 600 }).unwrap();
    }
    
    println!("\nAdaptive strategy results:");
    let final_penalties = state.economic_state.penalties.get(&1).unwrap_or(&0);
    let total_violations = state.economic_state.slashing_conditions.len();
    println!("  Final penalties for adaptive node: {}", final_penalties);
    println!("  Total violations recorded: {}", total_violations);
}

fn main() {
    println!("Advanced Byzantine Behaviors in Alpenglow Consensus");
    println!("==================================================");
    
    coalition_attack_example();
    network_partition_example();
    economic_attack_example();
    adaptive_strategy_example();
    
    println!("\n=== Summary ===");
    println!("This example demonstrated:");
    println!("1. Coordinated Byzantine coalitions with equivocation attacks");
    println!("2. Network partition attacks and recovery mechanisms");
    println!("3. Economic incentive-based attacks and slashing");
    println!("4. Adaptive Byzantine strategies that evolve over time");
    println!();
    println!("The advanced model provides comprehensive Byzantine fault tolerance");
    println!("analysis with realistic network conditions and economic incentives.");
}

#[cfg(test)]
mod example_tests {
    use super::*;
    
    #[test]
    fn test_coalition_example_setup() {
        let nodes = vec![1, 2, 3, 4, 5];
        let stake_distribution: HashMap<NodeId, StakeAmount> = nodes.iter()
            .map(|&node| (node, 100_000))
            .collect();
        
        let model = AlpenglowAdvancedModel::new(
            nodes.clone(),
            stake_distribution,
            NetworkConditions::default(),
            EconomicParams::default(),
        );
        
        let state = &model.init_states()[0];
        assert_eq!(state.nodes.len(), 5);
        assert_eq!(state.current_slot, 1);
    }
    
    #[test]
    fn test_network_conditions_setup() {
        let conditions = NetworkConditions {
            base_latency: 200,
            jitter: 100,
            packet_loss_rate: 0.15,
            corruption_rate: 0.05,
        };
        
        assert_eq!(conditions.base_latency, 200);
        assert_eq!(conditions.packet_loss_rate, 0.15);
    }
    
    #[test]
    fn test_economic_params_setup() {
        let params = EconomicParams {
            base_reward: 2000,
            penalty_multiplier: 2.5,
            reward_pool: 1_000_000,
            inflation_rate: 0.05,
            validator_reward_share: 0.8,
        };
        
        assert_eq!(params.base_reward, 2000);
        assert_eq!(params.reward_pool, 1_000_000);
    }
}