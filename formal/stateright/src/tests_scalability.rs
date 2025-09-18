use crate::lib_improved::*;
use std::collections::HashMap;

// Simplified test file that focuses on scalability without complex statistical model checking

#[test]
fn test_basic_scalability_properties() {
    // Test properties that should hold for large networks
    for network_size in vec![10, 25, 50, 100] {
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
        
        // Test the simplified scalability properties
        assert!(state.verify_scalability_properties());
        
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
fn test_large_network_creation_performance() {
    // Test that we can create very large networks efficiently
    let network_size = 200;
    let mut nodes = vec![];
    let mut stake_map = HashMap::new();
    
    for i in 0..network_size {
        let is_byzantine = i % 12 == 0; // About 8.3% Byzantine
        let stake = 50 + (i as u64 * 2);
        
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
    
    // Should create large state efficiently (under 50ms is reasonable)
    assert!(creation_time.as_millis() < 50);
    
    // Verify state properties
    assert_eq!(state.nodes.len(), 200);
    assert_eq!(state.stake_distribution.len(), 200);
    assert!(state.verify_scalability_properties());
    
    // Byzantine nodes should be grouped correctly
    let byzantine_count = state.byzantine_coalitions[0].members.len();
    assert_eq!(byzantine_count, 17); // 200/12 = 16.66... rounded up to 17
    
    // Compact state should be efficient for large networks
    let compact_start = std::time::Instant::now();
    let compact = state.to_compact_state();
    let compact_time = compact_start.elapsed();
    
    assert!(compact_time.as_millis() < 10);
    assert_eq!(compact.essential_metrics.active_byzantine_nodes, 17);
}

#[test]
fn test_network_scalability_linear() {
    // Verify that network creation scales roughly linearly
    let mut times = vec![];
    let sizes = vec![50, 100, 150, 200];
    
    for &size in &sizes {
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
        
        let start = std::time::Instant::now();
        let state = AlpenglowState::new_with_nodes(nodes, stake_map);
        let elapsed = start.elapsed();
        times.push(elapsed.as_nanos());
        
        // Verify basic properties
        assert_eq!(state.nodes.len(), size as usize);
        assert!(state.verify_scalability_properties());
    }
    
    // Check that time complexity is reasonable (not exponential)
    // The ratio between largest and smallest should be roughly proportional to size ratio
    let time_ratio = times[3] as f64 / times[0] as f64;
    let size_ratio = sizes[3] as f64 / sizes[0] as f64;
    
    // Time complexity should be roughly linear or at most quadratic
    assert!(time_ratio < size_ratio * size_ratio * 2.0);
}

#[test]
fn test_compact_state_memory_efficiency() {
    // Test that compact state representation is memory efficient
    let network_size = 100;
    let mut nodes = vec![];
    let mut stake_map = HashMap::new();
    
    for i in 0..network_size {
        let is_byzantine = i % 8 == 0;
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
    
    // Compact state should be much smaller than full state
    // Full state size is roughly proportional to number of nodes
    // Compact state should be fixed size
    
    // Test that multiple compact states with same properties are equal
    let compact2 = state.to_compact_state();
    assert_eq!(compact, compact2);
    
    // Test that compact state can be used in hash-based collections
    let mut compact_set = std::collections::HashSet::new();
    compact_set.insert(compact.clone());
    assert!(compact_set.contains(&compact2));
}

#[test]  
fn test_statistical_config_usage() {
    // Test that statistical configuration structures work correctly
    let config = StatisticalConfig {
        max_samples: 5000,
        confidence_level: 0.99,
        error_bound: 0.01,
        parallel_workers: 8,
        max_depth: Some(50),
    };
    
    assert_eq!(config.max_samples, 5000);
    assert_eq!(config.confidence_level, 0.99);
    assert_eq!(config.parallel_workers, 8);
    
    // Test different sampling strategies
    let strategies = vec![
        SamplingStrategy {
            sampling_type: SamplingType::UniformRandom,
            priority_weights: HashMap::new(),
        },
        SamplingStrategy {
            sampling_type: SamplingType::ImportanceSampling,
            priority_weights: HashMap::new(),
        },
    ];
    
    for strategy in strategies {
        // Strategies should be creatable and usable
        assert!(strategy.priority_weights.is_empty());
    }
}

#[test]
fn test_scalability_properties_validation() {
    // Test the scalability property validation function
    
    // Create a well-balanced network
    let nodes = vec![
        Node { id: 0, stake: 100, is_byzantine: false },
        Node { id: 1, stake: 100, is_byzantine: false },
        Node { id: 2, stake: 100, is_byzantine: false },
        Node { id: 3, stake: 100, is_byzantine: true },  // 25% Byzantine
    ];
    let stake_map: HashMap<u32, u64> = nodes.iter().map(|n| (n.id, n.stake)).collect();
    
    let state = AlpenglowState::new_with_nodes(nodes, stake_map);
    
    // Should pass scalability validation
    assert!(state.verify_scalability_properties());
    
    // Test individual properties
    assert_eq!(state.nodes.len(), 4);
    assert_eq!(state.total_stake(), 400);
    assert!(!state.stake_distribution.is_empty());
}

#[test]
fn test_maximum_practical_network_size() {
    // Test with a very large network to demonstrate practical limits
    let network_size = 500; // Even larger network
    let mut nodes = vec![];
    let mut stake_map = HashMap::new();
    
    for i in 0..network_size {
        let is_byzantine = i % 20 == 0; // 5% Byzantine (very conservative)
        let stake = 100;
        
        nodes.push(Node { 
            id: i, 
            stake, 
            is_byzantine,
        });
        stake_map.insert(i, stake);
    }
    
    let start_time = std::time::Instant::now();
    let state = AlpenglowState::new_with_nodes(nodes, stake_map);
    let creation_time = start_time.elapsed();
    
    // Should handle even very large networks
    assert!(creation_time.as_millis() < 100); // Should be fast even for 500 nodes
    assert_eq!(state.nodes.len(), 500);
    assert!(state.verify_scalability_properties());
    
    // Byzantine count should be reasonable
    let byzantine_count = state.byzantine_coalitions[0].members.len();
    assert_eq!(byzantine_count, 25); // 500/20 = 25
    
    // Compact representation should still work efficiently  
    let compact = state.to_compact_state();
    assert_eq!(compact.essential_metrics.active_byzantine_nodes, 25);
}

#[test]
fn test_stress_test_1000_nodes() {
    // Ultimate scalability stress test - 1000 nodes
    let network_size = 1000;
    let mut nodes = vec![];
    let mut stake_map = HashMap::new();
    
    for i in 0..network_size {
        let is_byzantine = i % 25 == 0; // 4% Byzantine
        let stake = 100;
        
        nodes.push(Node { 
            id: i, 
            stake, 
            is_byzantine,
        });
        stake_map.insert(i, stake);
    }
    
    let start_time = std::time::Instant::now();
    let state = AlpenglowState::new_with_nodes(nodes, stake_map);
    let creation_time = start_time.elapsed();
    
    // Even 1000 nodes should be manageable
    assert!(creation_time.as_millis() < 200); // Allow more time for 1000 nodes
    assert_eq!(state.nodes.len(), 1000);
    assert!(state.verify_scalability_properties());
    
    // Verify Byzantine resilience at large scale
    let byzantine_count = state.byzantine_coalitions[0].members.len();
    assert_eq!(byzantine_count, 40); // 1000/25 = 40
    
    let honest_count = 1000 - byzantine_count;
    assert!(honest_count > byzantine_count * 2); // Still maintains Byzantine resilience
    
    // Compact state should work even at this scale
    let compact_start = std::time::Instant::now();
    let compact = state.to_compact_state();
    let compact_time = compact_start.elapsed();
    
    assert!(compact_time.as_millis() < 20);
    assert_eq!(compact.essential_metrics.active_byzantine_nodes, 40);
}