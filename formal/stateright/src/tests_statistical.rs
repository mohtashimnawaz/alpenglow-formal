use crate::lib_improved::*;
use std::collections::HashMap;

#[test]
fn test_statistical_config_defaults() {
    let config = StatisticalConfig::default();
    
    assert_eq!(config.max_samples, 10000);
    assert_eq!(config.confidence_level, 0.95);
    assert_eq!(config.error_bound, 0.05);
    assert_eq!(config.parallel_workers, 4);
    assert_eq!(config.max_depth, Some(100));
}

#[test]
fn test_sampling_strategy_defaults() {
    let strategy = SamplingStrategy::default();
    
    assert_eq!(strategy.sampling_type, SamplingType::UniformRandom);
    assert!(strategy.priority_weights.is_empty());
}

#[test]
fn test_compact_state_representation() {
    let nodes = vec![
        Node { id: 0, stake: 100, is_byzantine: false },
        Node { id: 1, stake: 100, is_byzantine: false },
        Node { id: 2, stake: 100, is_byzantine: true },
    ];
    let stake_map: HashMap<u32, u64> = nodes.iter().map(|n| (n.id, n.stake)).collect();
    
    let state = AlpenglowState::new_with_nodes(nodes, stake_map);
    let compact = state.to_compact_state();
    
    // Should create consistent compact representation
    let compact2 = state.to_compact_state();
    assert_eq!(compact, compact2);
    assert_eq!(compact.essential_metrics.committed_blocks, 0);
}

#[test]
fn test_node_constructor() {
    let nodes = vec![
        Node { id: 0, stake: 100, is_byzantine: false },
        Node { id: 1, stake: 150, is_byzantine: true },
        Node { id: 2, stake: 200, is_byzantine: false },
    ];
    let stake_map: HashMap<u32, u64> = nodes.iter().map(|n| (n.id, n.stake)).collect();
    
    let state = AlpenglowState::new_with_nodes(nodes, stake_map);
    
    // Should have 3 nodes
    assert_eq!(state.nodes.len(), 3);
    assert!(state.nodes.contains(&0));
    assert!(state.nodes.contains(&1));
    assert!(state.nodes.contains(&2));
    
    // Should have correct stake distribution
    assert_eq!(state.stake_distribution[&0], 100);
    assert_eq!(state.stake_distribution[&1], 150);
    assert_eq!(state.stake_distribution[&2], 200);
    
    // Should have Byzantine coalition for node 1
    assert!(!state.byzantine_coalitions.is_empty());
    assert!(state.byzantine_coalitions[0].members.contains(&1));
}

#[test]
fn test_essential_metrics() {
    let metrics = EssentialMetrics {
        committed_blocks: 10,
        active_byzantine_nodes: 2,
        network_partitions: 1,
        average_latency: 50,
    };
    
    assert_eq!(metrics.committed_blocks, 10);
    assert_eq!(metrics.active_byzantine_nodes, 2);
    assert_eq!(metrics.network_partitions, 1);
    assert_eq!(metrics.average_latency, 50);
}

#[test]
fn test_compact_state_equality() {
    let metrics1 = EssentialMetrics {
        committed_blocks: 5,
        active_byzantine_nodes: 1,
        network_partitions: 0,
        average_latency: 25,
    };
    
    let compact1 = CompactState {
        consensus_hash: 12345,
        network_hash: 67890,
        byzantine_hash: 11111,
        essential_metrics: metrics1.clone(),
    };
    
    let compact2 = CompactState {
        consensus_hash: 12345,
        network_hash: 67890,
        byzantine_hash: 11111,
        essential_metrics: metrics1,
    };
    
    assert_eq!(compact1, compact2);
    
    // Test that Hash works (no panic means Hash is implemented)
    use std::collections::HashMap;
    let mut set = HashMap::new();
    set.insert(compact1.clone(), "test");
    assert!(set.contains_key(&compact2));
}

#[test]
fn test_large_network_state_creation() {
    // Test with larger network (50 nodes) - This demonstrates scalability
    let mut nodes = vec![];
    let mut stake_map = HashMap::new();
    
    for i in 0..50 {
        let is_byzantine = i % 10 == 0; // Every 10th node is Byzantine
        let stake = 100 + (i as u64 * 10);
        
        nodes.push(Node { 
            id: i, 
            stake, 
            is_byzantine,
        });
        stake_map.insert(i, stake);
    }
    
    let state = AlpenglowState::new_with_nodes(nodes, stake_map);
    
    // Should handle large networks efficiently
    assert_eq!(state.nodes.len(), 50);
    assert_eq!(state.stake_distribution.len(), 50);
    
    // Should have Byzantine coalitions for Byzantine nodes
    assert!(!state.byzantine_coalitions.is_empty());
    let byzantine_count = state.byzantine_coalitions[0].members.len();
    assert_eq!(byzantine_count, 5); // Every 10th node = 5 Byzantine nodes
    
    // Verify the state can be compacted efficiently
    let compact = state.to_compact_state();
    assert_eq!(compact.essential_metrics.active_byzantine_nodes, 5);
}

#[test]
fn test_very_large_network_performance() {
    // Test with even larger network (100 nodes) to demonstrate scalability
    let mut nodes = vec![];
    let mut stake_map = HashMap::new();
    
    for i in 0..100 {
        let is_byzantine = i % 15 == 0; // About 6.7% Byzantine nodes
        let stake = 50 + (i as u64 * 5);
        
        nodes.push(Node { 
            id: i, 
            stake, 
            is_byzantine,
        });
        stake_map.insert(i, stake);
    }
    
    // This should complete quickly, demonstrating efficient state creation
    let start_time = std::time::Instant::now();
    let state = AlpenglowState::new_with_nodes(nodes, stake_map);
    let creation_time = start_time.elapsed();
    
    // Should create large state efficiently (under 10ms is reasonable)
    assert!(creation_time.as_millis() < 10);
    
    // Verify state properties
    assert_eq!(state.nodes.len(), 100);
    assert_eq!(state.stake_distribution.len(), 100);
    
    // Byzantine nodes should be grouped correctly
    let byzantine_count = state.byzantine_coalitions[0].members.len();
    assert_eq!(byzantine_count, 7); // 100/15 = 6.66... rounded up to 7
    
    // Compact state should be efficient for large networks
    let compact_start = std::time::Instant::now();
    let compact = state.to_compact_state();
    let compact_time = compact_start.elapsed();
    
    assert!(compact_time.as_millis() < 5);
    assert_eq!(compact.essential_metrics.active_byzantine_nodes, 7);
}

#[test]
fn test_statistical_sampling_types() {
    let sampling_types = vec![
        SamplingType::UniformRandom,
        SamplingType::ImportanceSampling,
        SamplingType::StratifiedSampling,
        SamplingType::AdaptiveSampling,
    ];
    
    // All sampling types should be creatable
    for sampling_type in sampling_types {
        let strategy = SamplingStrategy {
            sampling_type: sampling_type.clone(),
            priority_weights: HashMap::new(),
        };
        assert_eq!(strategy.sampling_type, sampling_type);
    }
}

#[test]
fn test_network_scalability_metrics() {
    // Create different sized networks and measure key metrics
    let network_sizes = vec![10, 25, 50, 100];
    
    for size in network_sizes {
        let mut nodes = vec![];
        let mut stake_map = HashMap::new();
        
        for i in 0..size {
            let is_byzantine = i % 10 == 0;
            let stake = 100;
            
            nodes.push(Node { 
                id: i, 
                stake, 
                is_byzantine,
            });
            stake_map.insert(i, stake);
        }
        
        let state = AlpenglowState::new_with_nodes(nodes, stake_map);
        let compact = state.to_compact_state();
        
        // Verify scalability properties
        assert_eq!(state.nodes.len(), size as usize);
        
        // Debug: print actual byzantine node count
        let actual_byzantine = compact.essential_metrics.active_byzantine_nodes;
        // Calculate expected based on actual logic: i % 10 == 0 for i in 0..size
        let expected_byzantine = (0..size).filter(|&i| i % 10 == 0).count() as u32;
        
        if actual_byzantine != expected_byzantine {
            println!("Size: {}, Expected: {}, Actual: {}", size, expected_byzantine, actual_byzantine);
            println!("Byzantine coalitions: {:?}", state.byzantine_coalitions.len());
            for (i, coalition) in state.byzantine_coalitions.iter().enumerate() {
                println!("Coalition {}: members = {:?}", i, coalition.members);
            }
        }
        
        assert_eq!(actual_byzantine, expected_byzantine);
        
        // Network should scale linearly with node count
        let expected_stake = (size as u64) * 100;
        let actual_stake: u64 = state.stake_distribution.values().sum();
        assert_eq!(actual_stake, expected_stake);
    }
}

#[test]
fn test_compact_state_compression_efficiency() {
    // Test that compact state provides meaningful compression
    let nodes = vec![
        Node { id: 0, stake: 100, is_byzantine: false },
        Node { id: 1, stake: 200, is_byzantine: true },
        Node { id: 2, stake: 150, is_byzantine: false },
        Node { id: 3, stake: 300, is_byzantine: true },
    ];
    let stake_map: HashMap<u32, u64> = nodes.iter().map(|n| (n.id, n.stake)).collect();
    
    let state = AlpenglowState::new_with_nodes(nodes, stake_map);
    let compact = state.to_compact_state();
    
    // Compact state should contain essential information
    assert_eq!(compact.essential_metrics.committed_blocks, 0);
    assert_eq!(compact.essential_metrics.active_byzantine_nodes, 2);
    
    // Hash values should be deterministic
    let compact2 = state.to_compact_state();
    assert_eq!(compact.consensus_hash, compact2.consensus_hash);
    assert_eq!(compact.network_hash, compact2.network_hash);
    assert_eq!(compact.byzantine_hash, compact2.byzantine_hash);
}

#[test]
fn test_statistical_result_properties() {
    // Test statistical result structure for different scenarios
    let high_confidence = StatisticalResult {
        samples_taken: 10000,
        property_satisfied_count: 9500,
        estimated_probability: 0.95,
        confidence_interval: (0.94, 0.96),
        convergence_achieved: true,
    };
    
    let low_confidence = StatisticalResult {
        samples_taken: 100,
        property_satisfied_count: 60,
        estimated_probability: 0.6,
        confidence_interval: (0.5, 0.7),
        convergence_achieved: false,
    };
    
    // High confidence scenario
    assert!(high_confidence.convergence_achieved);
    assert!(high_confidence.estimated_probability > 0.9);
    assert!(high_confidence.confidence_interval.1 - high_confidence.confidence_interval.0 < 0.1);
    
    // Low confidence scenario  
    assert!(!low_confidence.convergence_achieved);
    assert!(low_confidence.confidence_interval.1 - low_confidence.confidence_interval.0 > 0.1);
}

// Simplified statistical property verification without full model checking
#[test]
fn test_basic_scalability_properties() {
    // Test properties that should hold for large networks
    for network_size in vec![20, 50, 100] {
        let mut nodes = vec![];
        let mut stake_map = HashMap::new();
        
        for i in 0..network_size {
            let is_byzantine = i % 8 == 0; // 12.5% Byzantine
            let stake = 100;
            
            nodes.push(Node { 
                id: i, 
                stake, 
                is_byzantine,
            });
            stake_map.insert(i, stake);
        }
        
        let state = AlpenglowState::new_with_nodes(nodes, stake_map);
        
        // Basic scalability properties
        assert!(state.nodes.len() == network_size as usize);
        assert!(state.total_stake() == (network_size as u64) * 100);
        
        // Byzantine resilience property (< 1/3 Byzantine)
        let byzantine_count = state.byzantine_coalitions[0].members.len();
        let honest_count = network_size as usize - byzantine_count;
        assert!(honest_count > byzantine_count * 2); // More than 2/3 honest
        
        // Network state should be initialized properly for large networks
        assert!(state.network_state.latency_model == LatencyModel::Constant { latency_ms: 50 });
        assert!(state.message_queue.pending_messages.is_empty());
    }
}

#[test]
fn test_parallel_processing_readiness() {
    // Test that our structures are ready for parallel processing
    let nodes = vec![
        Node { id: 0, stake: 100, is_byzantine: false },
        Node { id: 1, stake: 100, is_byzantine: true },
        Node { id: 2, stake: 100, is_byzantine: false },
        Node { id: 3, stake: 100, is_byzantine: false },
    ];
    let stake_map: HashMap<u32, u64> = nodes.iter().map(|n| (n.id, n.stake)).collect();
    
    let state = AlpenglowState::new_with_nodes(nodes, stake_map);
    
    // Test that state can be cloned (required for parallel processing)
    let state_clone = state.clone();
    assert_eq!(state.nodes.len(), state_clone.nodes.len());
    
    // Test that compact state works with threading requirements
    let compact = state.to_compact_state();
    let compact_clone = compact.clone();
    assert_eq!(compact, compact_clone);
    
    // Verify essential structures are thread-safe ready
    let config = StatisticalConfig::default();
    assert_eq!(config.parallel_workers, 4);
    assert_eq!(config.max_samples, 10000);
}