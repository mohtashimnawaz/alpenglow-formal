use crate::lib_improved::*;
use std::collections::HashMap;

#[test]
fn test_economic_state_initialization() {
    let nodes = vec![0, 1, 2];
    let stake_dist = HashMap::from([(0, 1000), (1, 1500), (2, 2000)]);
    
    let state = AlpenglowState::new(nodes.clone(), stake_dist.clone());
    
    // Should have initialized economic state
    assert_eq!(state.economic_state.rewards_pool, 3000); // 3 nodes * 1000
    assert_eq!(state.economic_state.total_slashed, 0);
    assert_eq!(state.economic_state.reward_rate, 0.05);
    assert_eq!(state.economic_state.slashing_rate, 0.1);
    assert_eq!(state.economic_state.validator_balances, stake_dist);
}

#[test] 
fn test_reward_calculation_and_distribution() {
    let nodes = vec![0, 1, 2];
    let stake_dist = HashMap::from([(0, 1000), (1, 1500), (2, 2000)]);
    
    let mut state = AlpenglowState::new(nodes, stake_dist);
    let participating_nodes = vec![0, 1, 2];
    
    // Calculate rewards for epoch 1
    let rewards = state.calculate_epoch_rewards(1, &participating_nodes);
    
    assert_eq!(rewards.epoch, 1);
    assert!(rewards.total_rewards > 0);
    assert_eq!(rewards.validator_rewards.len(), 3);
    assert_eq!(rewards.participation_rewards.len(), 3);
    assert_eq!(rewards.performance_bonuses.len(), 3); // All honest initially
    
    // Distribute rewards
    let initial_pool = state.economic_state.rewards_pool;
    let result = state.distribute_rewards(&rewards);
    assert!(result.is_ok());
    assert_eq!(state.economic_state.rewards_pool, initial_pool - rewards.total_rewards);
    
    // Check validator balances increased
    for (&node, &initial_stake) in &state.stake_distribution {
        let new_balance = state.economic_state.validator_balances[&node];
        assert!(new_balance >= initial_stake, "Node {} balance should increase", node);
    }
}

#[test]
fn test_slashing_detection_and_application() {
    let nodes = vec![0, 1, 2];
    let stake_dist = HashMap::from([(0, 1000), (1, 1500), (2, 2000)]);
    
    let mut state = AlpenglowState::new(nodes, stake_dist);
    
    // Create double voting evidence
    let vote1 = Vote {
        node: 1,
        slot: 2,
        block: 10,
        path: VotePath::Fast,
        stake: 1500,
    };
    
    let vote2 = Vote {
        node: 1,
        slot: 2,
        block: 11, // Different block, same slot
        path: VotePath::Fast,
        stake: 1500,
    };
    
    // Detect double voting
    let evidence = state.detect_double_voting(&vote1, &vote2);
    assert!(evidence.is_some());
    
    let evidence = evidence.unwrap();
    assert_eq!(evidence.violator, 1);
    assert_eq!(evidence.evidence_type, SlashingType::DoubleVoting);
    assert_eq!(evidence.severity, SlashingSeverity::Severe);
    
    // Apply slashing
    let initial_balance = state.economic_state.validator_balances[&1];
    let slash_result = state.apply_slashing(&evidence);
    assert!(slash_result.is_ok());
    
    let slashed_amount = slash_result.unwrap();
    let new_balance = state.economic_state.validator_balances[&1];
    
    assert_eq!(new_balance, initial_balance - slashed_amount);
    assert_eq!(state.economic_state.total_slashed, slashed_amount);
    assert_eq!(slashed_amount, (initial_balance as f64 * 0.30) as u64); // 30% for severe
    assert_eq!(state.economic_state.slashing_evidence.len(), 1);
}

#[test]
fn test_slashing_severities() {
    let nodes = vec![0, 1, 2, 3];
    let stake_dist = HashMap::from([(0, 1000), (1, 1000), (2, 1000), (3, 1000)]);
    
    let mut state = AlpenglowState::new(nodes, stake_dist);
    
    let severities = vec![
        (SlashingSeverity::Minor, 0.05),
        (SlashingSeverity::Moderate, 0.15),
        (SlashingSeverity::Severe, 0.30),
        (SlashingSeverity::Critical, 0.50),
    ];
    
    for (node_id, (severity, expected_rate)) in severities.iter().enumerate() {
        let evidence = SlashingEvidence {
            evidence_type: SlashingType::Equivocation,
            violator: node_id as u32,
            slot: 1,
            evidence_data: SlashingData::InvalidBlock {
                block: Block { id: 1, parent: 0 },
                violation: "Test violation".to_string(),
            },
            severity: severity.clone(),
            reporter: None,
            timestamp: 0,
        };
        
        let initial_balance = state.economic_state.validator_balances[&(node_id as u32)];
        let slash_result = state.apply_slashing(&evidence);
        assert!(slash_result.is_ok());
        
        let slashed_amount = slash_result.unwrap();
        let expected_slash = (initial_balance as f64 * expected_rate) as u64;
        assert_eq!(slashed_amount, expected_slash, "Wrong slash amount for {:?}", severity);
        
        // Critical slashing should mark as Byzantine
        if matches!(severity, SlashingSeverity::Critical) {
            assert!(matches!(state.status[&(node_id as u32)], NodeStatus::Byzantine(_)));
        }
    }
}

#[test]
fn test_stake_deposit_and_withdrawal() {
    let nodes = vec![0, 1];
    let stake_dist = HashMap::from([(0, 1000), (1, 1500)]);
    
    let mut state = AlpenglowState::new(nodes, stake_dist);
    
    // Test stake deposit
    let initial_balance = state.economic_state.validator_balances[&0];
    let initial_stake = state.stake_distribution[&0];
    
    let deposit_amount = 500;
    let deposit_action = AlpenglowAction::StakeDeposit { 
        node: 0, 
        amount: deposit_amount 
    };
    
    if let Some(new_state) = AlpenglowModel::new().next_state(&state, deposit_action) {
        assert_eq!(new_state.economic_state.validator_balances[&0], initial_balance + deposit_amount);
        assert_eq!(new_state.stake_distribution[&0], initial_stake + deposit_amount);
        state = new_state;
    }
    
    // Test stake withdrawal
    let withdrawal_amount = 200;
    let withdrawal_action = AlpenglowAction::StakeWithdrawal { 
        node: 0, 
        amount: withdrawal_amount 
    };
    
    if let Some(new_state) = AlpenglowModel::new().next_state(&state, withdrawal_action) {
        assert_eq!(new_state.economic_state.validator_balances[&0], 
                  initial_balance + deposit_amount - withdrawal_amount);
        assert_eq!(new_state.stake_distribution[&0], 
                  initial_stake + deposit_amount - withdrawal_amount);
    }
}

#[test]
fn test_reward_withdrawal() {
    let nodes = vec![0, 1];
    let stake_dist = HashMap::from([(0, 1000), (1, 1500)]);
    
    let mut state = AlpenglowState::new(nodes, stake_dist);
    
    // First add some rewards
    let rewards = RewardDistribution {
        epoch: 1,
        total_rewards: 500,
        validator_rewards: HashMap::from([(0, 300), (1, 200)]),
        performance_bonuses: HashMap::from([(0, 50), (1, 50)]),
        participation_rewards: HashMap::from([(0, 25), (1, 25)]),
    };
    
    state.distribute_rewards(&rewards).unwrap();
    let balance_after_rewards = state.economic_state.validator_balances[&0];
    
    // Now test withdrawal
    let withdrawal_amount = 100;
    let withdrawal_action = AlpenglowAction::WithdrawRewards { 
        node: 0, 
        amount: withdrawal_amount 
    };
    
    if let Some(new_state) = AlpenglowModel::new().next_state(&state, withdrawal_action) {
        assert_eq!(new_state.economic_state.validator_balances[&0], 
                  balance_after_rewards - withdrawal_amount);
    }
}

#[test]
fn test_economic_invariant_validation() {
    let nodes = vec![0, 1, 2];
    let stake_dist = HashMap::from([(0, 1000), (1, 1500), (2, 2000)]);
    
    let state = AlpenglowState::new(nodes, stake_dist);
    
    // Fresh state should have no violations
    let violations = state.validate_economic_invariants();
    assert!(violations.is_empty(), "Fresh state should have no violations: {:?}", violations);
    
    // Test with modified state that violates invariants
    let mut bad_state = state.clone();
    bad_state.economic_state.validator_balances.insert(0, 0); // Zero balance for active validator
    
    let violations = bad_state.validate_economic_invariants();
    assert!(!violations.is_empty(), "Should detect zero balance violation");
    assert!(violations.iter().any(|v| v.contains("zero balance")));
}

#[test] 
fn test_economic_game_theory_scenarios() {
    let nodes = vec![0, 1, 2, 3]; // 4 validators
    let stake_dist = HashMap::from([(0, 2000), (1, 1500), (2, 1000), (3, 500)]); // Different stake sizes
    
    let mut state = AlpenglowState::new(nodes, stake_dist);
    
    // Scenario 1: High-stake validator misbehaves (more to lose)
    let high_stake_evidence = SlashingEvidence {
        evidence_type: SlashingType::DoubleVoting,
        violator: 0, // Highest stake
        slot: 1,
        evidence_data: SlashingData::DoubleVote {
            vote1: Vote { node: 0, slot: 1, block: 10, path: VotePath::Fast, stake: 2000 },
            vote2: Vote { node: 0, slot: 1, block: 11, path: VotePath::Fast, stake: 2000 },
        },
        severity: SlashingSeverity::Severe,
        reporter: Some(1),
        timestamp: 100,
    };
    
    let initial_high_balance = state.economic_state.validator_balances[&0];
    state.apply_slashing(&high_stake_evidence).unwrap();
    let high_stake_loss = initial_high_balance - state.economic_state.validator_balances[&0];
    
    // Scenario 2: Low-stake validator misbehaves (less to lose)
    let low_stake_evidence = SlashingEvidence {
        evidence_type: SlashingType::DoubleVoting,
        violator: 3, // Lowest stake
        slot: 1,
        evidence_data: SlashingData::DoubleVote {
            vote1: Vote { node: 3, slot: 1, block: 10, path: VotePath::Fast, stake: 500 },
            vote2: Vote { node: 3, slot: 1, block: 11, path: VotePath::Fast, stake: 500 },
        },
        severity: SlashingSeverity::Severe,
        reporter: Some(1),
        timestamp: 200,
    };
    
    let initial_low_balance = state.economic_state.validator_balances[&3];
    state.apply_slashing(&low_stake_evidence).unwrap();
    let low_stake_loss = initial_low_balance - state.economic_state.validator_balances[&3];
    
    // High-stake validator should lose more in absolute terms
    assert!(high_stake_loss > low_stake_loss);
    
    // But same percentage (aligned incentives)
    let high_percentage = high_stake_loss as f64 / initial_high_balance as f64;
    let low_percentage = low_stake_loss as f64 / initial_low_balance as f64;
    assert!((high_percentage - low_percentage).abs() < 0.01); // Within 1%
    
    // Verify economic incentives are preserved
    assert_eq!(state.economic_state.total_slashed, high_stake_loss + low_stake_loss);
    assert_eq!(state.economic_state.slashing_evidence.len(), 2);
}

#[test]
fn test_validator_economics_at_scale() {
    // Test economics with larger validator set
    let mut nodes = vec![];
    let mut stake_dist = HashMap::new();
    
    for i in 0..20 {
        nodes.push(i);
        stake_dist.insert(i, 1000 + (i as u64 * 100)); // Varied stakes
    }
    
    let mut state = AlpenglowState::new(nodes.clone(), stake_dist);
    
    // Simulate epoch rewards
    let participating_nodes: Vec<_> = nodes.iter().take(15).copied().collect(); // 75% participation
    let rewards = state.calculate_epoch_rewards(1, &participating_nodes);
    
    assert_eq!(rewards.validator_rewards.len(), 15);
    assert!(rewards.total_rewards > 0);
    
    // Distribute rewards
    let initial_pool = state.economic_state.rewards_pool;
    state.distribute_rewards(&rewards).unwrap();
    
    // Pool should decrease
    assert!(state.economic_state.rewards_pool < initial_pool);
    
    // Participating validators should have increased balances
    for &node in &participating_nodes {
        let initial_stake = state.stake_distribution[&node];
        let current_balance = state.economic_state.validator_balances[&node];
        assert!(current_balance >= initial_stake);
    }
    
    // Non-participating validators should have unchanged balances
    for node in 15..20 {
        let initial_stake = state.stake_distribution[&node];
        let current_balance = state.economic_state.validator_balances[&node];
        assert_eq!(current_balance, initial_stake);
    }
}

#[test]
fn test_economic_attack_resistance() {
    let nodes = vec![0, 1, 2, 3];
    let stake_dist = HashMap::from([(0, 1000), (1, 1000), (2, 1000), (3, 1000)]);
    
    let mut state = AlpenglowState::new(nodes, stake_dist);
    
    // Simulate a coordinated attack by multiple validators
    let attackers = vec![0, 1]; // 50% of validators
    
    for &attacker in &attackers {
        let evidence = SlashingEvidence {
            evidence_type: SlashingType::LongRangeAttack,
            violator: attacker,
            slot: 1,
            evidence_data: SlashingData::NetworkAttack {
                attack_details: "Coordinated long-range attack".to_string(),
            },
            severity: SlashingSeverity::Critical, // Maximum punishment
            reporter: Some(2), // Honest validator reports
            timestamp: 100,
        };
        
        state.apply_slashing(&evidence).unwrap();
        
        // Attacker should be marked as Byzantine
        assert!(matches!(state.status[&attacker], NodeStatus::Byzantine(_)));
    }
    
    // Total slashing should be significant 
    let total_attack_stake: u64 = attackers.iter().map(|&id| state.stake_distribution[&id]).sum();
    let expected_min_slash = (total_attack_stake as f64 * 0.5) as u64; // 50% minimum
    assert!(state.economic_state.total_slashed >= expected_min_slash);
    
    // Honest validators should remain honest and keep their stake
    for honest_validator in vec![2, 3] {
        assert!(matches!(state.status[&honest_validator], NodeStatus::Honest));
        assert_eq!(state.economic_state.validator_balances[&honest_validator], 
                  state.stake_distribution[&honest_validator]);
    }
}