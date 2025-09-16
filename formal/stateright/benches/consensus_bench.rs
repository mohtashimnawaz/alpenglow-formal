use criterion::{black_box, criterion_group, criterion_main, Criterion};
use alpenglow_stateright::*;
use std::collections::HashMap;

fn create_benchmark_model(nodes: u32) -> AlpenglowState {
    let mut stake_distribution = HashMap::new();
    let stake_per_node = 1000 / nodes as u64;
    
    for i in 1..=nodes {
        stake_distribution.insert(i, stake_per_node);
    }
    
    AlpenglowState::new((1..=nodes).collect(), stake_distribution)
}

fn benchmark_model_checking_4_nodes(c: &mut Criterion) {
    c.bench_function("model_checking_4_nodes", |b| {
        b.iter(|| {
            let model = create_benchmark_model(4);
            model.checker()
                .spawn_bfs()
                .join()
                .assert_properties();
        })
    });
}

fn benchmark_model_checking_6_nodes(c: &mut Criterion) {
    c.bench_function("model_checking_6_nodes", |b| {
        b.iter(|| {
            let model = create_benchmark_model(6);
            model.checker()
                .spawn_bfs()
                .join()
                .assert_properties();
        })
    });
}

fn benchmark_state_generation(c: &mut Criterion) {
    c.bench_function("state_generation", |b| {
        let model = create_benchmark_model(4);
        let initial_state = model.init_states()[0].clone();
        
        b.iter(|| {
            let actions = model.actions(black_box(&initial_state));
            for action in actions.into_iter().take(10) {
                let _ = model.next_state(&initial_state, action);
            }
        })
    });
}

fn benchmark_vote_processing(c: &mut Criterion) {
    c.bench_function("vote_processing", |b| {
        let model = create_benchmark_model(4);
        let state = model.init_states()[0].clone();
        
        b.iter(|| {
            let action = AlpenglowAction::Vote {
                node: 1,
                slot: 1, 
                block: 1,
                path: VotePath::Fast,
            };
            let _ = model.next_state(black_box(&state), action);
        })
    });
}

fn benchmark_certificate_generation(c: &mut Criterion) {
    c.bench_function("certificate_generation", |b| {
        let model = create_benchmark_model(4);
        let mut state = model.init_states()[0].clone();
        
        // Pre-populate with votes
        for node in 1..=4 {
            let vote = Vote {
                node,
                slot: 1,
                block: 1,
                path: VotePath::Fast,
                timestamp: 0,
                stake: 250,
            };
            state.votes.get_mut(&node).unwrap().get_mut(&1).unwrap().push(vote);
        }
        
        b.iter(|| {
            let action = AlpenglowAction::Certify {
                slot: 1,
                path: VotePath::Fast,
            };
            let _ = model.next_state(black_box(&state), action);
        })
    });
}

criterion_group!(
    benches,
    benchmark_model_checking_4_nodes,
    benchmark_model_checking_6_nodes, 
    benchmark_state_generation,
    benchmark_vote_processing,
    benchmark_certificate_generation
);

criterion_main!(benches);