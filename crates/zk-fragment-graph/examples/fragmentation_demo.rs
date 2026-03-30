//! Simple fragmentation demonstration using only Week 1 code

use zk_fragment_graph::prelude::*;
use std::collections::{HashMap, HashSet};

fn main() {
    println!("=== ZK-FRAGMENT: Manual Fragmentation Demo ===\n");
    
    // Build a larger circuit
    let graph = create_chain_circuit(20);
    
    println!("Circuit: {} constraints, {} wires", 
        graph.constraints.len(), 
        graph.wires.len()
    );
    
    // Analyze it
    let cut_analysis = find_cut_vertices(&graph);
    println!("\nCut vertices found: {}", cut_analysis.cut_vertices.len());
    
    // Manually fragment using topological levels
    println!("\n=== Manual Fragmentation ===\n");
    
    let levels = group_by_depth(&graph).unwrap();
    println!("Circuit has {} depth levels", levels.len());
    
    // Create 4 fragments by grouping levels
    let target_fragments = 4;
    let levels_per_fragment = (levels.len() + target_fragments - 1) / target_fragments;
    
    let mut fragments: Vec<Vec<ConstraintId>> = vec![Vec::new(); target_fragments];
    
    for (level_idx, level) in levels.iter().enumerate() {
        let fragment_idx = (level_idx / levels_per_fragment).min(target_fragments - 1);
        fragments[fragment_idx].extend(level.iter().copied());
    }
    
    // Display fragments
    println!("\nFragments created:");
    for (i, fragment) in fragments.iter().enumerate() {
        println!("  Fragment {}: {} constraints", i, fragment.len());
    }
    
    // Find boundary wires between fragments
    println!("\n=== Boundary Analysis ===\n");
    
    let boundaries = find_boundaries(&graph, &fragments);
    
    println!("Total boundary wires: {}", boundaries.len());
    println!("\nBoundary details:");
    for (wire_id, from_frag, to_frag) in boundaries.iter() {
        println!("  Wire {} : Fragment {} → Fragment {}", wire_id, from_frag, to_frag);
    }
    
    // Calculate overhead
    let total_constraints: usize = fragments.iter().map(|f| f.len()).sum();
    let boundary_overhead = boundaries.len() as f64 / total_constraints as f64;
    
    println!("\n=== Metrics ===\n");
    println!("Total constraints: {}", total_constraints);
    println!("Boundary wires: {}", boundaries.len());
    println!("Boundary overhead: {:.1}%", boundary_overhead * 100.0);
    
    // Estimate parallelism
    let fragment_sizes: Vec<_> = fragments.iter().map(|f| f.len()).collect();
    let max_size = *fragment_sizes.iter().max().unwrap();
    let min_size = *fragment_sizes.iter().min().unwrap();
    let avg_size = fragment_sizes.iter().sum::<usize>() as f64 / fragments.len() as f64;
    
    println!("\nFragment sizes:");
    println!("  Max: {}", max_size);
    println!("  Min: {}", min_size);
    println!("  Avg: {:.1}", avg_size);
    println!("  Balance: {:.1}%", (min_size as f64 / max_size as f64) * 100.0);
    
    println!("\nEstimated speedup: {:.2}x (with {} parallel cores)", 
        fragments.len() as f64 * 0.7, // Rough estimate
        fragments.len()
    );
    
    
}

// Helper function to find boundary wires
fn find_boundaries(
    graph: &ConstraintGraph,
    fragments: &[Vec<ConstraintId>],
) -> Vec<(WireId, usize, usize)> {
    // Build constraint -> fragment mapping
    let mut constraint_to_fragment: HashMap<ConstraintId, usize> = HashMap::new();
    for (frag_idx, fragment) in fragments.iter().enumerate() {
        for &constraint in fragment {
            constraint_to_fragment.insert(constraint, frag_idx);
        }
    }
    
    let mut boundaries = Vec::new();
    let mut seen: HashSet<(WireId, usize, usize)> = HashSet::new();
    
    // For each wire, check if it crosses fragment boundaries
    for wire in graph.wires.values() {
        if let Some(producer) = wire.producer {
            let source_frag = match constraint_to_fragment.get(&producer) {
                Some(&f) => f,
                None => continue,
            };
            
            for &consumer in &wire.consumers {
                let target_frag = match constraint_to_fragment.get(&consumer) {
                    Some(&f) => f,
                    None => continue,
                };
                
                if source_frag != target_frag {
                    let boundary = (wire.id, source_frag, target_frag);
                    if !seen.contains(&boundary) {
                        boundaries.push(boundary);
                        seen.insert(boundary);
                    }
                }
            }
        }
    }
    
    boundaries.sort_by_key(|b| (b.1, b.2, b.0 .0));
    boundaries
}