use crate::*;
use stateright::Model;
use std::collections::{HashMap, HashSet};

#[test]
fn test_basic_message_sending() {
    let nodes = vec![1, 2, 3];
    let mut stake_distribution = HashMap::new();
    for &node in &nodes {
        stake_distribution.insert(node, 100);
    }
    
    let state = AlpenglowState::new(nodes, stake_distribution);
    let model = state.clone();
    
    // Test sending a heartbeat message
    let send_action = AlpenglowAction::SendMessage {
        from: 1,
        to: 2,
        content: MessageContent::Heartbeat { sequence: 42 },
        priority: MessagePriority::Normal,
    };
    
    let new_state = model.next_state(&state, send_action).unwrap();
    
    // Message should be in pending queue
    assert_eq!(new_state.message_queue.pending_messages.len(), 1);
    assert_eq!(new_state.message_queue.pending_messages[0].from, 1);
    assert_eq!(new_state.message_queue.pending_messages[0].to, 2);
    assert!(new_state.message_queue.pending_messages[0].scheduled_delivery_time > state.global_time);
}

#[test]
fn test_message_delivery() {
    let nodes = vec![1, 2, 3];
    let mut stake_distribution = HashMap::new();
    for &node in &nodes {
        stake_distribution.insert(node, 100);
    }
    
    let mut state = AlpenglowState::new(nodes, stake_distribution);
    let model = state.clone();
    
    // Send a vote message
    let vote = Vote {
        node: 1,
        slot: 1,
        block: 0,
        path: VotePath::Fast,
        stake: 100,
    };
    
    let send_action = AlpenglowAction::SendMessage {
        from: 1,
        to: 2,
        content: MessageContent::Vote(vote.clone()),
        priority: MessagePriority::Critical,
    };
    
    let state_with_pending = model.next_state(&state, send_action).unwrap();
    assert_eq!(state_with_pending.message_queue.pending_messages.len(), 1);
    
    // Advance time to trigger delivery
    let message_id = state_with_pending.message_queue.pending_messages[0].id;
    state.global_time = state_with_pending.message_queue.pending_messages[0].scheduled_delivery_time;
    
    let deliver_action = AlpenglowAction::DeliverMessage { message_id };
    let final_state = model.next_state(&state_with_pending, deliver_action).unwrap();
    
    // Message should be delivered and removed from pending
    assert_eq!(final_state.message_queue.pending_messages.len(), 0);
    assert_eq!(final_state.message_queue.delivered_messages.len(), 1);
    
    // Vote should be delivered to receiving node
    let delivered_votes = &final_state.votes[&2][&1];
    assert!(delivered_votes.iter().any(|v| v.node == vote.node && v.block == vote.block));
}

#[test]
fn test_latency_models() {
    let nodes = vec![1, 2];
    let mut stake_distribution = HashMap::new();
    stake_distribution.insert(1, 100);
    stake_distribution.insert(2, 100);
    
    let state = AlpenglowState::new(nodes, stake_distribution);
    let model = state.clone();
    
    // Test constant latency
    let constant_latency = model.calculate_latency(&state, 1, 2);
    assert_eq!(constant_latency, 50); // Default constant latency
    
    // Test uniform latency model
    let mut state_uniform = state.clone();
    state_uniform.network_state.latency_model = LatencyModel::Uniform { 
        min_ms: 10, 
        max_ms: 100 
    };
    let uniform_latency = model.calculate_latency(&state_uniform, 1, 2);
    assert!(uniform_latency >= 10 && uniform_latency <= 100);
    
    // Test normal distribution model
    let mut state_normal = state.clone();
    state_normal.network_state.latency_model = LatencyModel::Normal { 
        mean_ms: 50, 
        std_dev_ms: 10 
    };
    let normal_latency = model.calculate_latency(&state_normal, 1, 2);
    assert!(normal_latency > 0); // Should be positive
    
    // Test realistic model
    let mut state_realistic = state.clone();
    state_realistic.network_state.latency_model = LatencyModel::Realistic { 
        base_latency_ms: 20,
        distance_factor: 5,
        congestion_multiplier: 10,
    };
    let realistic_latency = model.calculate_latency(&state_realistic, 1, 2);
    assert!(realistic_latency >= 20); // Should be at least base latency
}

#[test]
fn test_packet_loss() {
    let nodes = vec![1, 2, 3];
    let mut stake_distribution = HashMap::new();
    for &node in &nodes {
        stake_distribution.insert(node, 100);
    }
    
    let mut state = AlpenglowState::new(nodes, stake_distribution);
    let model = state.clone();
    
    // Inject packet loss failure
    let failure = NetworkFailure {
        failure_type: FailureType::PacketLoss { loss_rate: 1.0 }, // 100% loss
        start_time: 0,
        duration: 100,
        affected_nodes: vec![1, 2],
        severity: 1.0,
    };
    
    let inject_action = AlpenglowAction::InjectNetworkFailure { failure };
    let state_with_failure = model.next_state(&state, inject_action).unwrap();
    
    // Try to send a message during packet loss
    let send_action = AlpenglowAction::SendMessage {
        from: 1,
        to: 2,
        content: MessageContent::Heartbeat { sequence: 1 },
        priority: MessagePriority::Normal,
    };
    
    let final_state = model.next_state(&state_with_failure, send_action).unwrap();
    
    // Message should be dropped due to packet loss
    assert_eq!(final_state.message_queue.pending_messages.len(), 0);
}

#[test]
fn test_network_partition_blocks_messages() {
    let nodes = vec![1, 2, 3, 4];
    let mut stake_distribution = HashMap::new();
    for &node in &nodes {
        stake_distribution.insert(node, 100);
    }
    
    let state = AlpenglowState::new(nodes, stake_distribution);
    let model = state.clone();
    
    // Create network partition
    let mut partition_a = HashSet::new();
    partition_a.insert(1);
    partition_a.insert(2);
    let mut partition_b = HashSet::new();
    partition_b.insert(3);
    partition_b.insert(4);
    
    let partition_action = AlpenglowAction::NetworkPartition {
        nodes_a: partition_a,
        nodes_b: partition_b,
    };
    
    let partitioned_state = model.next_state(&state, partition_action).unwrap();
    
    // Try to send message across partition (should be dropped)
    let cross_partition_send = AlpenglowAction::SendMessage {
        from: 1, // In partition A
        to: 3,   // In partition B
        content: MessageContent::Heartbeat { sequence: 1 },
        priority: MessagePriority::Normal,
    };
    
    let state_after_cross_send = model.next_state(&partitioned_state, cross_partition_send).unwrap();
    assert_eq!(state_after_cross_send.message_queue.pending_messages.len(), 0);
    
    // Try to send message within partition (should succeed)
    let within_partition_send = AlpenglowAction::SendMessage {
        from: 1, // In partition A
        to: 2,   // Also in partition A
        content: MessageContent::Heartbeat { sequence: 2 },
        priority: MessagePriority::Normal,
    };
    
    let state_after_within_send = model.next_state(&partitioned_state, within_partition_send).unwrap();
    assert_eq!(state_after_within_send.message_queue.pending_messages.len(), 1);
}

#[test]
fn test_latency_spike() {
    let nodes = vec![1, 2];
    let mut stake_distribution = HashMap::new();
    stake_distribution.insert(1, 100);
    stake_distribution.insert(2, 100);
    
    let state = AlpenglowState::new(nodes, stake_distribution);
    let model = state.clone();
    
    // Inject latency spike
    let failure = NetworkFailure {
        failure_type: FailureType::LatencySpike { multiplier: 5.0 },
        start_time: 0,
        duration: 50,
        affected_nodes: vec![1, 2],
        severity: 0.8,
    };
    
    let inject_action = AlpenglowAction::InjectNetworkFailure { failure };
    let state_with_failure = model.next_state(&state, inject_action).unwrap();
    
    // Send message during latency spike
    let send_action = AlpenglowAction::SendMessage {
        from: 1,
        to: 2,
        content: MessageContent::Heartbeat { sequence: 1 },
        priority: MessagePriority::Normal,
    };
    
    let final_state = model.next_state(&state_with_failure, send_action).unwrap();
    
    // Message should be sent but with higher latency
    assert_eq!(final_state.message_queue.pending_messages.len(), 1);
    let message = &final_state.message_queue.pending_messages[0];
    let delivery_delay = message.scheduled_delivery_time - message.send_time;
    
    // Should have increased latency due to spike (5x multiplier)
    assert!(delivery_delay > 50); // Original latency was 50ms, should be much higher now
}

#[test]
fn test_congestion_effects() {
    let nodes = vec![1, 2, 3];
    let mut stake_distribution = HashMap::new();
    for &node in &nodes {
        stake_distribution.insert(node, 100);
    }
    
    let state = AlpenglowState::new(nodes, stake_distribution);
    let model = state.clone();
    
    // Simulate high congestion
    let congestion_action = AlpenglowAction::SimulateCongestion {
        links: vec![(1, 2)],
        intensity: 0.95, // Very high congestion
    };
    
    let congested_state = model.next_state(&state, congestion_action).unwrap();
    
    // Send message on congested link
    let send_action = AlpenglowAction::SendMessage {
        from: 1,
        to: 2,
        content: MessageContent::Heartbeat { sequence: 1 },
        priority: MessagePriority::Normal,
    };
    
    let final_state = model.next_state(&congested_state, send_action).unwrap();
    
    // Message should be sent but with congestion delay
    assert_eq!(final_state.message_queue.pending_messages.len(), 1);
    let message = &final_state.message_queue.pending_messages[0];
    let delivery_delay = message.scheduled_delivery_time - message.send_time;
    
    // Should have congestion-induced delay
    assert!(delivery_delay > 50); // Should be higher than base latency due to congestion
}

#[test]
fn test_message_priority_ordering() {
    let nodes = vec![1, 2];
    let mut stake_distribution = HashMap::new();
    stake_distribution.insert(1, 100);
    stake_distribution.insert(2, 100);
    
    let state = AlpenglowState::new(nodes, stake_distribution);
    let model = state.clone();
    
    // Send messages with different priorities
    let critical_action = AlpenglowAction::SendMessage {
        from: 1,
        to: 2,
        content: MessageContent::Vote(Vote {
            node: 1,
            slot: 1,
            block: 0,
            path: VotePath::Fast,
            stake: 100,
        }),
        priority: MessagePriority::Critical,
    };
    
    let normal_action = AlpenglowAction::SendMessage {
        from: 1,
        to: 2,
        content: MessageContent::Heartbeat { sequence: 1 },
        priority: MessagePriority::Normal,
    };
    
    let state1 = model.next_state(&state, critical_action).unwrap();
    let state2 = model.next_state(&state1, normal_action).unwrap();
    
    // Both messages should be queued
    assert_eq!(state2.message_queue.pending_messages.len(), 2);
    
    // Messages should have different priorities
    let priorities: Vec<_> = state2.message_queue.pending_messages
        .iter()
        .map(|msg| &msg.priority)
        .collect();
    
    assert!(priorities.contains(&&MessagePriority::Critical));
    assert!(priorities.contains(&&MessagePriority::Normal));
}

#[test]
fn test_network_failure_recovery() {
    let nodes = vec![1, 2];
    let mut stake_distribution = HashMap::new();
    stake_distribution.insert(1, 100);
    stake_distribution.insert(2, 100);
    
    let state = AlpenglowState::new(nodes, stake_distribution);
    let model = state.clone();
    
    // Inject failure
    let failure = NetworkFailure {
        failure_type: FailureType::LinkFailure { from: 1, to: 2 },
        start_time: 0,
        duration: 50,
        affected_nodes: vec![1, 2],
        severity: 1.0,
    };
    
    let inject_action = AlpenglowAction::InjectNetworkFailure { failure };
    let failed_state = model.next_state(&state, inject_action).unwrap();
    
    assert_eq!(failed_state.network_state.failure_injections.len(), 1);
    
    // Recover from failure
    let recover_action = AlpenglowAction::RecoverFromFailure { failure_index: 0 };
    let recovered_state = model.next_state(&failed_state, recover_action).unwrap();
    
    assert_eq!(recovered_state.network_state.failure_injections.len(), 0);
    
    // Now messages should be able to get through
    let send_action = AlpenglowAction::SendMessage {
        from: 1,
        to: 2,
        content: MessageContent::Heartbeat { sequence: 1 },
        priority: MessagePriority::Normal,
    };
    
    let final_state = model.next_state(&recovered_state, send_action).unwrap();
    assert_eq!(final_state.message_queue.pending_messages.len(), 1);
}

#[test]
fn test_dynamic_latency_model_updates() {
    let nodes = vec![1, 2];
    let mut stake_distribution = HashMap::new();
    stake_distribution.insert(1, 100);
    stake_distribution.insert(2, 100);
    
    let state = AlpenglowState::new(nodes, stake_distribution);
    let model = state.clone();
    
    // Start with constant latency
    assert!(matches!(
        state.network_state.latency_model,
        LatencyModel::Constant { latency_ms: 50 }
    ));
    
    // Update to realistic model
    let update_action = AlpenglowAction::UpdateLatencyModel {
        new_model: LatencyModel::Realistic {
            base_latency_ms: 30,
            distance_factor: 2,
            congestion_multiplier: 5,
        },
    };
    
    let updated_state = model.next_state(&state, update_action).unwrap();
    
    // Model should be updated
    assert!(matches!(
        updated_state.network_state.latency_model,
        LatencyModel::Realistic { .. }
    ));
    
    // Latency calculation should use new model
    let new_latency = model.calculate_latency(&updated_state, 1, 2);
    assert_ne!(new_latency, 50); // Should be different from original constant latency
}

#[test]
fn test_bandwidth_adjustment() {
    let nodes = vec![1, 2, 3];
    let mut stake_distribution = HashMap::new();
    for &node in &nodes {
        stake_distribution.insert(node, 100);
    }
    
    let state = AlpenglowState::new(nodes, stake_distribution);
    let model = state.clone();
    
    // Initially no bandwidth limits
    assert!(state.network_state.bandwidth_limits.is_empty());
    
    // Set bandwidth limit
    let bandwidth_action = AlpenglowAction::AdjustBandwidth {
        from: 1,
        to: 2,
        new_bandwidth: 1000, // 1KB/s
    };
    
    let limited_state = model.next_state(&state, bandwidth_action).unwrap();
    
    // Bandwidth limit should be set
    assert_eq!(limited_state.network_state.bandwidth_limits.len(), 1);
    assert_eq!(limited_state.network_state.bandwidth_limits[&(1, 2)], 1000);
}