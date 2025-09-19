use crate::lib_improved::*;
use std::collections::HashMap;

#[test]
fn test_rotor_erasure_coding_creation() {
    let nodes = vec![0, 1, 2, 3];
    let stake_dist = HashMap::from([(0, 1000), (1, 1500), (2, 2000), (3, 500)]);
    
    let state = AlpenglowState::new(nodes, stake_dist);
    
    let block = Block { id: 1, parent: 0 };
    let erasure_block = state.create_erasure_coded_block(block, 1.5); // 50% redundancy
    
    assert_eq!(erasure_block.block.id, 1);
    assert_eq!(erasure_block.redundancy_level, 1.5);
    assert_eq!(erasure_block.required_chunks, 10); // Base chunks needed
    assert!(erasure_block.chunks.len() >= 10); // At least base chunks
    
    // Verify chunk integrity
    for chunk in &erasure_block.chunks {
        assert_eq!(chunk.block_id, 1);
        assert!(!chunk.data.is_empty());
    }
}

#[test]
fn test_rotor_stake_weighted_relay_selection() {
    let nodes = vec![0, 1, 2, 3];
    let stake_dist = HashMap::from([
        (0, 1000), // 20% stake
        (1, 1500), // 30% stake  
        (2, 2000), // 40% stake
        (3, 500),  // 10% stake
    ]);
    
    let state = AlpenglowState::new(nodes, stake_dist);
    
    let block = Block { id: 1, parent: 0 };
    let erasure_block = state.create_erasure_coded_block(block, 1.0);
    
    let relay_nodes = state.select_relay_nodes(1, &erasure_block);
    
    assert!(!relay_nodes.is_empty());
    
    // Higher stake nodes should get more chunks (roughly proportional)
    let node_2_chunks = relay_nodes.iter()
        .find(|r| r.node_id == 2)
        .map(|r| r.assigned_chunks.len())
        .unwrap_or(0);
    
    let node_3_chunks = relay_nodes.iter()
        .find(|r| r.node_id == 3)
        .map(|r| r.assigned_chunks.len())
        .unwrap_or(0);
    
    // Node 2 (40% stake) should have more chunks than node 3 (10% stake)
    assert!(node_2_chunks >= node_3_chunks);
    
    // Verify relay node properties
    for relay in &relay_nodes {
        assert!(relay.stake_weight > 0);
        assert!(relay.reliability_score > 0.0);
        assert!(!relay.assigned_chunks.is_empty());
    }
}

#[test]
fn test_rotor_chunk_reconstruction() {
    let nodes = vec![0, 1, 2];
    let stake_dist = HashMap::from([(0, 1000), (1, 1000), (2, 1000)]);
    
    let mut state = AlpenglowState::new(nodes, stake_dist);
    
    let block = Block { id: 1, parent: 0 };
    let erasure_block = state.create_erasure_coded_block(block, 0.5);
    
    // Initially, block cannot be reconstructed (no chunks available)
    assert!(!state.can_reconstruct_block(1));
    
    // Add the erasure block and set up chunks
    state.erasure_coded_blocks.insert(1, erasure_block.clone());
    
    // Simulate having enough chunks available from different nodes
    for i in 0..erasure_block.required_chunks {
        state.chunk_availability
            .entry((1, i as u32))
            .or_insert_with(std::collections::HashSet::new)
            .insert(i as u32 % 3); // Distribute across nodes
    }
    
    // Now block should be reconstructable
    assert!(state.can_reconstruct_block(1));
}

#[test]
fn test_leader_rotation_deterministic() {
    let nodes = vec![0, 1, 2, 3];
    let stake_dist = HashMap::from([(0, 1000), (1, 1500), (2, 2000), (3, 500)]);
    
    let mut state = AlpenglowState::new(nodes, stake_dist);
    
    // Test deterministic leader selection
    let leader_1 = state.get_leader_for_slot(1);
    let leader_2 = state.get_leader_for_slot(2);
    let leader_3 = state.get_leader_for_slot(3);
    
    // Leaders should be deterministic for same slot
    assert_eq!(leader_1, state.get_leader_for_slot(1));
    assert_eq!(leader_2, state.get_leader_for_slot(2));
    
    // Test rotation
    state.rotate_leader(2);
    assert_eq!(state.leader_rotation.current_slot, 2);
    assert_eq!(state.leader_rotation.leader_history.len(), 2); // Initial + rotated
    
    // Verify leader schedule changes with stake weighting
    let schedule = state.generate_leader_schedule_for_window(1);
    assert_eq!(schedule.len(), 4);
    assert!(schedule.contains(&0));
    assert!(schedule.contains(&1));
    assert!(schedule.contains(&2));
    assert!(schedule.contains(&3));
}

#[test]
fn test_windowing_system() {
    let nodes = vec![0, 1, 2];
    let stake_dist = HashMap::from([(0, 1000), (1, 1000), (2, 1000)]);
    
    let mut state = AlpenglowState::new(nodes, stake_dist);
    
    // Initial window
    assert_eq!(state.current_window.window_start, 1);
    assert_eq!(state.current_window.window_size, 10);
    assert_eq!(state.current_window.finality_depth, 2);
    
    // Test window update
    state.update_window(11, 15, 3);
    
    assert_eq!(state.current_window.window_start, 11);
    assert_eq!(state.current_window.window_size, 15);
    assert_eq!(state.current_window.finality_depth, 3);
    
    // Leader schedule should be updated for new window
    assert_eq!(state.current_window.leader_schedule.len(), 3);
}

#[test]
fn test_bounded_finalization_time() {
    let nodes = vec![0, 1, 2];
    let stake_dist = HashMap::from([(0, 1000), (1, 1000), (2, 1000)]);
    
    let mut state = AlpenglowState::new(nodes, stake_dist);
    
    // Test with valid finalization time
    let slot = 1u32;
    let slot_start_time = slot as u64 * 1000; // 1 second per slot
    let valid_finalization_time = slot_start_time + 400; // Within 500ms bound
    
    state.finalization_times.insert(slot, valid_finalization_time);
    assert!(state.check_finalization_time_bounds(slot));
    
    // Test with invalid finalization time
    let invalid_finalization_time = slot_start_time + 1200; // Exceeds min(500, 2*1000) = 500ms
    state.finalization_times.insert(slot, invalid_finalization_time);
    assert!(!state.check_finalization_time_bounds(slot));
}

#[test]
fn test_rotor_erasure_actions() {
    let nodes = vec![0, 1, 2];
    let stake_dist = HashMap::from([(0, 1000), (1, 1000), (2, 1000)]);
    
    let state = AlpenglowState::new(nodes, stake_dist);
    let model = AlpenglowModel::new();
    
    let block = Block { id: 1, parent: 0 };
    let erasure_block = state.create_erasure_coded_block(block, 1.0);
    
    // Test PropagateErasureBlock action
    let action = AlpenglowAction::PropagateErasureBlock { 
        node: 0, 
        erasure_block: erasure_block.clone() 
    };
    
    let new_state = model.next_state(&state, action);
    assert!(new_state.is_some());
    
    let new_state = new_state.unwrap();
    assert!(new_state.erasure_coded_blocks.contains_key(&1));
    assert!(!new_state.relay_assignments.is_empty());
    
    // Test chunk propagation action
    let chunk = erasure_block.chunks[0].clone();
    let chunk_action = AlpenglowAction::PropagateChunk {
        node: 0,
        chunk,
        target_nodes: vec![1, 2],
    };
    
    let chunk_result = model.next_state(&new_state, chunk_action);
    assert!(chunk_result.is_some());
    
    let chunk_state = chunk_result.unwrap();
    assert!(chunk_state.chunk_availability.contains_key(&(1, 0)));
}

#[test]
fn test_leader_rotation_actions() {
    let nodes = vec![0, 1, 2];
    let stake_dist = HashMap::from([(0, 1000), (1, 1000), (2, 1000)]);
    
    let state = AlpenglowState::new(nodes, stake_dist);
    let model = AlpenglowModel::new();
    
    // Test leader rotation action
    let rotation_action = AlpenglowAction::RotateLeader { 
        new_leader: 1, 
        slot: 2 
    };
    
    let new_state = model.next_state(&state, rotation_action);
    assert!(new_state.is_some());
    
    let new_state = new_state.unwrap();
    assert_eq!(new_state.leader_rotation.current_slot, 2);
    assert_eq!(new_state.leader_rotation.leader_history.len(), 2);
    
    // Test window update action
    let window_action = AlpenglowAction::UpdateWindow {
        slot: 11,
        window_size: 20,
        finality_depth: 5,
    };
    
    let window_result = model.next_state(&new_state, window_action);
    assert!(window_result.is_some());
    
    let window_state = window_result.unwrap();
    assert_eq!(window_state.current_window.window_start, 11);
    assert_eq!(window_state.current_window.window_size, 20);
}

#[test]
fn test_complete_alpenglow_protocol_properties() {
    let nodes = vec![0, 1, 2, 3, 4]; // 5 validators
    let stake_dist = HashMap::from([
        (0, 2000), (1, 1500), (2, 1500), (3, 1000), (4, 1000)
    ]);
    
    let mut state = AlpenglowState::new(nodes, stake_dist);
    
    // Add some erasure coded blocks
    let block = Block { id: 1, parent: 0 };
    let erasure_block = state.create_erasure_coded_block(block, 1.0);
    state.erasure_coded_blocks.insert(1, erasure_block.clone());
    
    // Set up chunk availability for reconstruction
    for i in 0..erasure_block.required_chunks {
        state.chunk_availability
            .entry((1, i as u32))
            .or_insert_with(std::collections::HashSet::new)
            .insert(i as u32 % 5);
    }
    
    // Add leader rotation history
    for slot in 1..=15 {
        state.rotate_leader(slot);
    }
    
    // Add valid finalization times
    state.finalization_times.insert(1, 1400); // Within bounds
    state.finalization_times.insert(2, 2450); // Within bounds
    
    // Test all properties
    use stateright::Model;
    let properties = state.properties();
    
    assert!(properties.len() >= 8); // Should have all properties including new ones
    
    // Verify property names
    let property_names: Vec<&str> = properties.iter().map(|p| p.name).collect();
    assert!(property_names.contains(&"bounded_finalization_time"));
    assert!(property_names.contains(&"erasure_block_availability")); 
    assert!(property_names.contains(&"leader_rotation_fairness"));
    
    // Test that all properties evaluate successfully (don't panic)
    for property in &properties {
        let result = (property.condition)(&state, &state);
        // Properties should evaluate without error (result can be true or false)
        println!("Property '{}': {}", property.name, result);
    }
}