pub mod cut_vertex;
pub mod balanced;
pub mod memory_aware;
pub mod config;

pub use cut_vertex::*;
pub use balanced::*;
pub use memory_aware::*;
pub use config::*;

use crate::fragment::{FragmentSpec, FragmentId, FragmentationResult, BoundaryWire};
use zk_fragment_graph::{ConstraintGraph, ConstraintId, WireId};
use std::collections::{HashMap, HashSet};

/// Trait for fragmentation strategies
pub trait FragmentationStrategy {
    /// Fragment the constraint graph
    fn fragment(&self, graph: &ConstraintGraph, config: &FragmentationConfig) -> Result<FragmentationResult, FragmentationError>;
    
    /// Strategy name for logging
    fn name(&self) -> &'static str;
}

#[derive(Debug, thiserror::Error)]
pub enum FragmentationError {
    #[error("Graph is too small to fragment (need at least {0} constraints)")]
    TooSmall(usize),
    
    #[error("Failed to partition graph: {0}")]
    PartitionFailed(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Graph validation failed: {0}")]
    GraphValidation(String),
}

/// Build fragment specifications from a partition assignment
pub fn build_fragments_from_partition(
    graph: &ConstraintGraph,
    partition: &HashMap<ConstraintId, usize>,
    num_partitions: usize,
) -> Result<FragmentationResult, FragmentationError> {
    // Group constraints by partition
    let mut partition_constraints: Vec<Vec<ConstraintId>> = vec![Vec::new(); num_partitions];
    
    for (&constraint_id, &partition_id) in partition {
        if partition_id < num_partitions {
            partition_constraints[partition_id].push(constraint_id);
        }
    }
    
    // Create fragments
    let mut fragments: Vec<FragmentSpec> = Vec::new();
    
    for (i, constraints) in partition_constraints.into_iter().enumerate() {
        if !constraints.is_empty() {
            let fragment = FragmentSpec::new(FragmentId(i), constraints);
            fragments.push(fragment);
        }
    }
    
    // Compute boundaries
    compute_boundaries(graph, &mut fragments);
    
    // Compute dependencies
    compute_dependencies(&mut fragments);
    
    // Compute execution order
    let execution_order = compute_execution_order(&fragments)?;
    
    // Compute metrics
    let total_boundary_wires = fragments.iter()
        .map(|f| f.output_boundaries.len())
        .sum();
    
    let (dependency_depth, max_parallelism) = compute_dependency_metrics(&fragments);
    
    Ok(FragmentationResult {
        fragments,
        total_boundary_wires,
        dependency_depth,
        max_parallelism,
        execution_order,
    })
}

/// Compute boundary wires between fragments
fn compute_boundaries(graph: &ConstraintGraph, fragments: &mut Vec<FragmentSpec>) {
    // Build constraint -> fragment mapping
    let mut constraint_to_fragment: HashMap<ConstraintId, FragmentId> = HashMap::new();
    for fragment in fragments.iter() {
        for &constraint in &fragment.constraints {
            constraint_to_fragment.insert(constraint, fragment.id);
        }
    }
    
    // For each wire, check if it crosses fragment boundaries
    let mut output_boundary_counters: HashMap<FragmentId, usize> = HashMap::new();
    let mut input_boundary_counters: HashMap<FragmentId, usize> = HashMap::new();
    
    for wire in graph.wires.values() {
        if let Some(producer) = wire.producer {
            let source_fragment = match constraint_to_fragment.get(&producer) {
                Some(&f) => f,
                None => continue,
            };
            
            for &consumer in &wire.consumers {
                let target_fragment = match constraint_to_fragment.get(&consumer) {
                    Some(&f) => f,
                    None => continue,
                };
                
                if source_fragment != target_fragment {
                    // This is a boundary wire
                    let source_index = *output_boundary_counters.entry(source_fragment).or_insert(0);
                    let target_index = *input_boundary_counters.entry(target_fragment).or_insert(0);
                    
                    let boundary = BoundaryWire {
                        wire_id: wire.id,
                        source_fragment,
                        target_fragment,
                        source_index,
                        target_index,
                    };
                    
                    // Add to source fragment's outputs
                    if let Some(source_frag) = fragments.iter_mut().find(|f| f.id == source_fragment) {
                        if !source_frag.output_boundaries.iter().any(|b| b.wire_id == wire.id && b.target_fragment == target_fragment) {
                            source_frag.output_boundaries.push(boundary.clone());
                            *output_boundary_counters.get_mut(&source_fragment).unwrap() += 1;
                        }
                    }
                    
                    // Add to target fragment's inputs
                    if let Some(target_frag) = fragments.iter_mut().find(|f| f.id == target_fragment) {
                                                if !target_frag.input_boundaries.iter().any(|b| b.wire_id == wire.id && b.source_fragment == source_fragment) {
                            target_frag.input_boundaries.push(boundary);
                            *input_boundary_counters.get_mut(&target_fragment).unwrap() += 1;
                        }
                    }
                }
            }
        }
    }
    
    // Classify internal wires and public IO for each fragment
    for fragment in fragments.iter_mut() {
        let constraint_set: HashSet<_> = fragment.constraints.iter().copied().collect();
        
        for &constraint_id in &fragment.constraints {
            if let Some(constraint) = graph.constraints.get(&constraint_id) {
                // Check input wires
                for &wire_id in &constraint.input_wires {
                    if let Some(wire) = graph.wires.get(&wire_id) {
                        match wire.role {
                            zk_fragment_graph::WireRole::PublicInput => {
                                if !fragment.public_inputs.contains(&wire_id) {
                                    fragment.public_inputs.push(wire_id);
                                }
                            }
                            _ => {
                                // Check if internal (produced within this fragment)
                                if let Some(producer) = wire.producer {
                                    if constraint_set.contains(&producer) {
                                        if !fragment.internal_wires.contains(&wire_id) {
                                            fragment.internal_wires.push(wire_id);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Check output wires
                for &wire_id in &constraint.output_wires {
                    if let Some(wire) = graph.wires.get(&wire_id) {
                        if matches!(wire.role, zk_fragment_graph::WireRole::PublicOutput) {
                            if !fragment.public_outputs.contains(&wire_id) {
                                fragment.public_outputs.push(wire_id);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Compute fragment dependencies based on boundaries
fn compute_dependencies(fragments: &mut Vec<FragmentSpec>) {
    // Build a map of fragment IDs to indices for efficient lookup
    let fragment_ids: Vec<FragmentId> = fragments.iter().map(|f| f.id).collect();
    
    for i in 0..fragments.len() {
        let mut deps = HashSet::new();
        
        for boundary in &fragments[i].input_boundaries {
            deps.insert(boundary.source_fragment);
        }
        
        fragments[i].dependencies = deps.into_iter().collect();
        fragments[i].dependencies.sort_by_key(|f| f.0);
    }
    
    // Compute dependents (reverse of dependencies)
    for i in 0..fragments.len() {
        let current_id = fragments[i].id;
        let deps = fragments[i].dependencies.clone();
        
        for dep_id in deps {
            if let Some(dep_frag) = fragments.iter_mut().find(|f| f.id == dep_id) {
                if !dep_frag.dependents.contains(&current_id) {
                    dep_frag.dependents.push(current_id);
                }
            }
        }
    }
}

/// Compute topological execution order of fragments
fn compute_execution_order(fragments: &[FragmentSpec]) -> Result<Vec<FragmentId>, FragmentationError> {
    let mut in_degree: HashMap<FragmentId, usize> = HashMap::new();
    let mut order = Vec::new();
    
    // Initialize in-degrees
    for fragment in fragments {
        in_degree.insert(fragment.id, fragment.dependencies.len());
    }
    
    // Find all fragments with no dependencies
    let mut queue: std::collections::VecDeque<FragmentId> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&id, _)| id)
        .collect();
    
    while let Some(id) = queue.pop_front() {
        order.push(id);
        
        // Find fragment and reduce in-degree of dependents
        if let Some(fragment) = fragments.iter().find(|f| f.id == id) {
            for &dependent in &fragment.dependents {
                if let Some(deg) = in_degree.get_mut(&dependent) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(dependent);
                    }
                }
            }
        }
    }
    
    if order.len() != fragments.len() {
        return Err(FragmentationError::PartitionFailed(
            "Circular dependency detected in fragments".to_string()
        ));
    }
    
    Ok(order)
}

/// Compute dependency depth and max parallelism
fn compute_dependency_metrics(fragments: &[FragmentSpec]) -> (usize, usize) {
    if fragments.is_empty() {
        return (0, 0);
    }
    
    // Compute depth of each fragment
    let mut depths: HashMap<FragmentId, usize> = HashMap::new();
    
    fn compute_depth(
        id: FragmentId,
        fragments: &[FragmentSpec],
        depths: &mut HashMap<FragmentId, usize>,
    ) -> usize {
        if let Some(&d) = depths.get(&id) {
            return d;
        }
        
        let fragment = match fragments.iter().find(|f| f.id == id) {
            Some(f) => f,
            None => return 0,
        };
        
        let depth = if fragment.dependencies.is_empty() {
            0
        } else {
            fragment.dependencies.iter()
                .map(|&dep| compute_depth(dep, fragments, depths) + 1)
                .max()
                .unwrap_or(0)
        };
        
        depths.insert(id, depth);
        depth
    }
    
    for fragment in fragments {
        compute_depth(fragment.id, fragments, &mut depths);
    }
    
    let max_depth = depths.values().copied().max().unwrap_or(0);
    
    // Compute max parallelism (max fragments at any level)
    let mut level_counts: HashMap<usize, usize> = HashMap::new();
    for &depth in depths.values() {
        *level_counts.entry(depth).or_insert(0) += 1;
    }
    
    let max_parallelism = level_counts.values().copied().max().unwrap_or(1);
    
    (max_depth, max_parallelism)
}