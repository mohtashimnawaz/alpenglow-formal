use crate::*;
use stateright::Model;
use std::collections::HashMap;

#[test]
fn test_selective_equivocation() {
    let nodes = vec![1, 2, 3, 4];
    let mut stake_distribution = HashMap::new();
    
    // Node 1 has high stake, others have low stake
    stake_distribution.insert(1, 500); // High stake
    stake_distribution.insert(2, 100);
    stake_distribution.insert(3, 100);
    stake_distribution.insert(4, 100);
    
    let mut state = AlpenglowState::new(nodes, stake_distribution);
    
    // Make node 1 Byzantine with selective equivocation strategy
    state.status.insert(1, NodeStatus::Byzantine(ByzantineStrategy::SelectiveEquivocation {
        min_stake_threshold: 300,
        target_slots: vec![1, 3], // Only target specific slots
    }));
    
    let model = state.clone();
    
    // Test Byzantine vote action on targeted slot
    let byzantine_action = AlpenglowAction::ByzantineVote {
        node: 1,
        strategy: ByzantineStrategy::SelectiveEquivocation {
            min_stake_threshold: 300,
            target_slots: vec![1, 3],
        },
        slot: 1, // This is a target slot
    };
    
    let new_state = model.next_state(&state, byzantine_action).unwrap();
    
    // Node 1 should have cast multiple votes for slot 1 (equivocation on target slot)
    let node1_slot1_votes = &new_state.votes[&1][&1];
    assert!(node1_slot1_votes.len() >= 2, "High-stake node should equivocate on target slot");
    
    // Test on non-targeted slot
    let byzantine_action_non_target = AlpenglowAction::ByzantineVote {
        node: 1,
        strategy: ByzantineStrategy::SelectiveEquivocation {
            min_stake_threshold: 300,
            target_slots: vec![1, 3],
        },
        slot: 2, // This is NOT a target slot
    };
    
    let new_state2 = model.next_state(&state, byzantine_action_non_target).unwrap();
    
    // Node 1 should behave honestly on non-target slot
    let node1_slot2_votes = &new_state2.votes[&1][&2];
    assert!(node1_slot2_votes.len() <= 1, "Node should behave honestly on non-target slot");
}

#[test]
fn test_adaptive_behavior() {
    let nodes = vec![1, 2, 3];
    let mut stake_distribution = HashMap::new();
    stake_distribution.insert(1, 100);
    stake_distribution.insert(2, 100);
    stake_distribution.insert(3, 100);
    
    let mut state = AlpenglowState::new(nodes, stake_distribution);
    
    // Set up adaptive Byzantine strategy
    state.status.insert(1, NodeStatus::Byzantine(ByzantineStrategy::AdaptiveBehavior {
        primary_strategy: Box::new(ByzantineStrategy::Equivocation),
        fallback_strategy: Box::new(ByzantineStrategy::WithholdVotes),
        adaptation_threshold: 2,
    }));
    
    // Simulate timeout conditions to trigger adaptation
    state.timeouts.get_mut(&1).unwrap().get_mut(&1).unwrap().count = 3; // Above threshold
    
    let model = state.clone();
    
    let byzantine_action = AlpenglowAction::ByzantineVote {
        node: 1,
        strategy: ByzantineStrategy::AdaptiveBehavior {
            primary_strategy: Box::new(ByzantineStrategy::Equivocation),
            fallback_strategy: Box::new(ByzantineStrategy::WithholdVotes),
            adaptation_threshold: 2,
        },
        slot: 1,
    };
    
    let new_state = model.next_state(&state, byzantine_action).unwrap();
    
    // With high timeout count, should use fallback strategy (WithholdVotes)
    let node1_votes = &new_state.votes[&1][&1];
    assert_eq!(node1_votes.len(), 0, "Node should withhold votes due to adaptation");
}

#[test]
fn test_coalition_formation() {
    let nodes = vec![1, 2, 3, 4, 5];
    let mut stake_distribution = HashMap::new();
    for &node in &nodes {
        stake_distribution.insert(node, 100);
    }
    
    let mut state = AlpenglowState::new(nodes, stake_distribution);
    
    // Make multiple nodes Byzantine
    state.status.insert(1, NodeStatus::Byzantine(ByzantineStrategy::Equivocation));
    state.status.insert(2, NodeStatus::Byzantine(ByzantineStrategy::Equivocation));
    state.status.insert(3, NodeStatus::Byzantine(ByzantineStrategy::RandomVotes));
    
    let model = state.clone();
    
    // Test coalition formation
    let coalition_action = AlpenglowAction::FormCoalition {
        members: vec![1, 2, 3],
        strategy: CoalitionAttackType::SplitVote {
            target_blocks: vec![0, 1, 2],
        },
    };
    
    let new_state = model.next_state(&state, coalition_action).unwrap();
    
    // Verify coalition was formed
    assert_eq!(new_state.byzantine_coalitions.len(), 1, "Coalition should be formed");
    assert_eq!(new_state.byzantine_coalitions[0].members.len(), 3, "Coalition should have 3 members");
    assert!(new_state.coalition_state.contains_key(&0), "Coalition state should be tracked");
    assert!(new_state.coalition_state[&0].active, "Coalition should be active");
}

#[test]
fn test_coalition_coordination() {
    let nodes = vec![1, 2, 3, 4];
    let mut stake_distribution = HashMap::new();
    for &node in &nodes {
        stake_distribution.insert(node, 100);
    }
    
    let mut state = AlpenglowState::new(nodes, stake_distribution);
    
    // Set up existing coalition
    state.byzantine_coalitions.push(ByzantineCoalition {
        members: vec![1, 2],
        strategy: CoalitionAttackType::StrategicTargeting {
            high_priority_slots: vec![1, 2],
            disruption_threshold: 0.7,
        },
        coordination_history: Vec::new(),
        total_stake: 200,
        formation_time: 0,
    });
    
    state.coalition_state.insert(0, CoalitionState {
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
    
    let model = state.clone();
    
    // Test coalition coordination
    let coordination_action = AlpenglowAction::CoordinateAttack {
        coalition_index: 0,
        target_slot: 1,
    };
    
    let new_state = model.next_state(&state, coordination_action).unwrap();
    
    // Verify coordination occurred
    assert_eq!(
        new_state.coalition_state[&0].current_phase,
        AttackPhase::Execution,
        "Coalition should move to execution phase"
    );
    
    assert!(
        !new_state.byzantine_coalitions[0].coordination_history.is_empty(),
        "Coordination history should be recorded"
    );
}

#[test]
fn test_timing_attack() {
    let nodes = vec![1, 2, 3];
    let mut stake_distribution = HashMap::new();
    for &node in &nodes {
        stake_distribution.insert(node, 100);
    }
    
    let mut state = AlpenglowState::new(nodes, stake_distribution);
    state.status.insert(1, NodeStatus::Byzantine(ByzantineStrategy::TimingAttack {
        delay_votes: true,
        max_delay: 500,
        target_path: Some(VotePath::Fast),
    }));
    
    let model = state.clone();
    let initial_time = state.global_time;
    
    let byzantine_action = AlpenglowAction::ByzantineVote {
        node: 1,
        strategy: ByzantineStrategy::TimingAttack {
            delay_votes: true,
            max_delay: 500,
            target_path: Some(VotePath::Fast),
        },
        slot: 1,
    };
    
    let new_state = model.next_state(&state, byzantine_action).unwrap();
    
    // Time should have advanced due to delay
    assert!(new_state.global_time > initial_time, "Time should advance due to timing attack");
    
    // Vote should still be cast but with delay
    let node1_votes = &new_state.votes[&1][&1];
    assert!(node1_votes.len() > 0, "Vote should be cast after delay");
    assert_eq!(node1_votes[0].path, VotePath::Fast, "Should target Fast path");
}

#[test]
fn test_stake_based_attack() {
    let nodes = vec![1, 2, 3];
    let mut stake_distribution = HashMap::new();
    stake_distribution.insert(1, 1000); // High stake attacker
    stake_distribution.insert(2, 100);
    stake_distribution.insert(3, 100);
    
    let mut state = AlpenglowState::new(nodes, stake_distribution);
    state.status.insert(1, NodeStatus::Byzantine(ByzantineStrategy::StakeBasedAttack {
        reserve_stake_for_critical_slots: true,
        activation_threshold: 500,
        min_profit_margin: 50,
    }));
    
    let model = state.clone();
    
    // Test on critical slot (slot % 3 == 0)
    let byzantine_action_critical = AlpenglowAction::ByzantineVote {
        node: 1,
        strategy: ByzantineStrategy::StakeBasedAttack {
            reserve_stake_for_critical_slots: true,
            activation_threshold: 500,
            min_profit_margin: 50,
        },
        slot: 3, // Critical slot (3 % 3 == 0)
    };
    
    let new_state = model.next_state(&state, byzantine_action_critical).unwrap();
    
    // Should cast multiple votes on critical slots for maximum disruption
    let node1_critical_votes = &new_state.votes[&1][&3];
    assert!(node1_critical_votes.len() >= 2, "Should cast multiple votes on critical slots");
    
    // Test on regular slot
    let byzantine_action_regular = AlpenglowAction::ByzantineVote {
        node: 1,
        strategy: ByzantineStrategy::StakeBasedAttack {
            reserve_stake_for_critical_slots: true,
            activation_threshold: 500,
            min_profit_margin: 50,
        },
        slot: 2, // Regular slot
    };
    
    let new_state2 = model.next_state(&state, byzantine_action_regular).unwrap();
    
    // Should be more conservative on regular slots
    let node1_regular_votes = &new_state2.votes[&1][&2];
    assert_eq!(node1_regular_votes.len(), 1, "Should be conservative on regular slots");
}

#[test]
fn test_strategy_adaptation_actions() {
    let nodes = vec![1, 2, 3];
    let mut stake_distribution = HashMap::new();
    for &node in &nodes {
        stake_distribution.insert(node, 100);
    }
    
    let mut state = AlpenglowState::new(nodes, stake_distribution);
    
    // Make nodes Byzantine to enable adaptation actions
    state.status.insert(1, NodeStatus::Byzantine(ByzantineStrategy::Equivocation));
    state.status.insert(2, NodeStatus::Byzantine(ByzantineStrategy::WithholdVotes));
    
    let model = state.clone();
    
    // Get available actions
    let mut actions = Vec::new();
    model.actions(&state, &mut actions);
    
    // Should have adaptation actions for Byzantine nodes
    let adaptation_actions: Vec<_> = actions.iter()
        .filter(|action| matches!(action, AlpenglowAction::AdaptStrategy { .. }))
        .collect();
    
    assert!(!adaptation_actions.is_empty(), "Should have strategy adaptation actions");
    
    // Test adaptation action
    if let AlpenglowAction::AdaptStrategy { node, new_strategy, reason: _ } = &adaptation_actions[0] {
        let adapt_action = adaptation_actions[0].clone();
        let new_state = model.next_state(&state, adapt_action).unwrap();
        
        // Node should have new strategy
        if let NodeStatus::Byzantine(updated_strategy) = &new_state.status[node] {
            assert_ne!(*updated_strategy, ByzantineStrategy::Equivocation, "Strategy should change");
        }
    }
}