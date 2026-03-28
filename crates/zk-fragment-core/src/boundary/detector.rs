use crate::fragment::{FragmentSpec, FragmentId, BoundaryWire};
use zk_fragment_graph::{ConstraintGraph, ConstraintId, WireId};
use std::collections::{HashMap, HashSet};

/// Information about boundary wires between fragments
#[derive(Debug, Clone)]
pub struct BoundaryAnalysis {
    /// All boundary wires
    pub boundaries: Vec<BoundaryWire>,
    /// Boundary wires grouped by source fragment
    pub by_source: HashMap<FragmentId, Vec<BoundaryWire>>,
    /// Boundary wires grouped by target fragment
    pub by_target: HashMap<FragmentId, Vec<BoundaryWire>>,
    /// Total number of boundary wires
    pub total_count: usize,
    /// Wires with multiple targets
    pub multi_target_wires: Vec<(WireId, Vec<FragmentId>)>,
}

/// Detect all boundary wires between fragments
pub fn detect_boundaries(
    graph: &ConstraintGraph,
    fragments: &[FragmentSpec],
) -> BoundaryAnalysis {
    // Build constraint -> fragment mapping
    let mut constraint_to_fragment: HashMap<ConstraintId, FragmentId> = HashMap::new();
    for fragment in fragments {
        for &constraint in &fragment.constraints {
            constraint_to_fragment.insert(constraint, fragment.id);
        }
    }
    
    let mut boundaries: Vec<BoundaryWire> = Vec::new();
    let mut by_source: HashMap<FragmentId, Vec<BoundaryWire>> = HashMap::new();
    let mut by_target: HashMap<FragmentId, Vec<BoundaryWire>> = HashMap::new();
    let mut wire_targets: HashMap<WireId, HashSet<FragmentId>> = HashMap::new();
    
    let mut source_indices: HashMap<FragmentId, usize> = HashMap::new();
    let mut target_indices: HashMap<FragmentId, usize> = HashMap::new();
    
    // Find boundary wires
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
                    // Track multi-target wires
                    wire_targets.entry(wire.id).or_default().insert(target_fragment);
                    
                    // Check if we already have this boundary
                    let exists = boundaries.iter().any(|b| {
                        b.wire_id == wire.id && 
                        b.source_fragment == source_fragment && 
                        b.target_fragment == target_fragment
                    });
                    
                    if !exists {
                        let source_index = *source_indices.entry(source_fragment).or_insert(0);
                        let target_index = *target_indices.entry(target_fragment).or_insert(0);
                        
                        let boundary = BoundaryWire {
                            wire_id: wire.id,
                            source_fragment,
                            target_fragment,
                            source_index,
                            target_index,
                        };
                        
                        boundaries.push(boundary.clone());
                        by_source.entry(source_fragment).or_default().push(boundary.clone());
                        by_target.entry(target_fragment).or_default().push(boundary);
                        
                        *source_indices.get_mut(&source_fragment).unwrap() += 1;
                        *target_indices.get_mut(&target_fragment).unwrap() += 1;
                    }
                }
            }
        }
    }
    
    // Find multi-target wires
    let multi_target_wires: Vec<_> = wire_targets.into_iter()
        .filter(|(_, targets)| targets.len() > 1)
        .map(|(wire, targets)| (wire, targets.into_iter().collect()))
        .collect();
    
    BoundaryAnalysis {
        total_count: boundaries.len(),
        boundaries,
        by_source,
        by_target,
        multi_target_wires,
    }
}

/// Compute boundary statistics
#[derive(Debug, Clone)]
pub struct BoundaryStats {
    /// Total boundary wires
    pub total: usize,
    /// Average boundaries per fragment
    pub avg_per_fragment: f64,
    /// Maximum output boundaries from any fragment
    pub max_output: usize,
    /// Maximum input boundaries to any fragment
    pub max_input: usize,
    /// Number of multi-target wires
    pub multi_target_count: usize,
    /// Boundary overhead ratio
    pub overhead_ratio: f64,
}

pub fn compute_boundary_stats(
    analysis: &BoundaryAnalysis,
    fragments: &[FragmentSpec],
    total_constraints: usize,
) -> BoundaryStats {
    let max_output = analysis.by_source.values()
        .map(|v| v.len())
        .max()
        .unwrap_or(0);
    
    let max_input = analysis.by_target.values()
        .map(|v| v.len())
        .max()
        .unwrap_or(0);
    
    let avg_per_fragment = if !fragments.is_empty() {
        analysis.total_count as f64 / fragments.len() as f64
    } else {
        0.0
    };
    
    let overhead_ratio = if total_constraints > 0 {
        analysis.total_count as f64 / total_constraints as f64
    } else {
        0.0
    };
    
    BoundaryStats {
        total: analysis.total_count,
        avg_per_fragment,
        max_output,
        max_input,
        multi_target_count: analysis.multi_target_wires.len(),
        overhead_ratio,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zk_fragment_graph::{Wire, Constraint, ConstraintType};

    fn create_test_graph_and_fragments() -> (zk_fragment_graph::ConstraintGraph, Vec<FragmentSpec>) {
        let mut graph = zk_fragment_graph::ConstraintGraph::new();
        
        // Create wires
        for i in 0..4 {
            graph.add_wire(Wire::internal(WireId(i)));
        }
        
        // Create constraints
        // Fragment 0: C0 -> produces W0
        // Fragment 1: C1 (uses W0) -> produces W1, C2 (uses W1) -> produces W2
        let c0 = Constraint::new(ConstraintId(0), ConstraintType::Add, vec![], vec![WireId(0)]);
        let c1 = Constraint::new(ConstraintId(1), ConstraintType::Mul, vec![WireId(0)], vec![WireId(1)]);
        let c2 = Constraint::new(ConstraintId(2), ConstraintType::Add, vec![WireId(1)], vec![WireId(2)]);
        
        graph.add_constraint(c0);
        graph.add_constraint(c1);
        graph.add_constraint(c2);
        
        // Set up wire relationships
        graph.wires.get_mut(&WireId(0)).unwrap().producer = Some(ConstraintId(0));
        graph.wires.get_mut(&WireId(0)).unwrap().consumers.push(ConstraintId(1));
        graph.wires.get_mut(&WireId(1)).unwrap().producer = Some(ConstraintId(1));
        graph.wires.get_mut(&WireId(1)).unwrap().consumers.push(ConstraintId(2));
        
        graph.build_edges_from_wires();
        
        let fragments = vec![
            FragmentSpec::new(FragmentId(0), vec![ConstraintId(0)]),
            FragmentSpec::new(FragmentId(1), vec![ConstraintId(1), ConstraintId(2)]),
        ];
        
        (graph, fragments)
    }

    #[test]
    fn test_detect_boundaries() {
        let (graph, fragments) = create_test_graph_and_fragments();
        let analysis = detect_boundaries(&graph, &fragments);
        
        // Should detect W0 as boundary from F0 to F1
        assert_eq!(analysis.total_count, 1);
        assert!(analysis.boundaries.iter().any(|b| b.wire_id == WireId(0)));
    }
}