use crate::fragment::{FragmentSpec, FragmentId};
use crate::boundary::WireRoutingTable;
use zk_fragment_graph::{ConstraintGraph, ConstraintId, WireId};
use std::collections::{HashMap, HashSet};

/// Partitioned witness data for a fragment
#[derive(Debug, Clone)]
pub struct FragmentWitness {
    /// Fragment ID
    pub fragment_id: FragmentId,
    /// Wire values indexed by local index
    pub values: Vec<u64>,
    /// Mapping from wire ID to local index
    pub wire_to_index: HashMap<WireId, usize>,
    /// Boundary input values (received from other fragments)
    pub boundary_inputs: Vec<(WireId, usize)>,
    /// Boundary output values (sent to other fragments)
    pub boundary_outputs: Vec<(WireId, usize)>,
}

/// Witness partitioner
#[derive(Debug)]
pub struct WitnessPartitioner {
    /// The constraint graph
    graph: ConstraintGraph,
    /// Fragment specifications
    fragments: Vec<FragmentSpec>,
    /// Wire routing table
    routing: WireRoutingTable,
}

impl WitnessPartitioner {
    pub fn new(
        graph: ConstraintGraph,
        fragments: Vec<FragmentSpec>,
        routing: WireRoutingTable,
    ) -> Self {
        Self {
            graph,
            fragments,
            routing,
        }
    }
    
    /// Partition a full witness into fragment witnesses
    pub fn partition(&self, full_witness: &HashMap<WireId, u64>) -> Vec<FragmentWitness> {
        self.fragments.iter()
            .map(|fragment| self.partition_for_fragment(fragment, full_witness))
            .collect()
    }
    
    /// Create witness for a single fragment
    fn partition_for_fragment(
        &self,
        fragment: &FragmentSpec,
        full_witness: &HashMap<WireId, u64>,
    ) -> FragmentWitness {
        let mut wire_to_index: HashMap<WireId, usize> = HashMap::new();
        let mut values: Vec<u64> = Vec::new();
        let mut boundary_inputs: Vec<(WireId, usize)> = Vec::new();
        let mut boundary_outputs: Vec<(WireId, usize)> = Vec::new();
        
        // Collect all wires needed by this fragment
        let needed_wires = self.collect_fragment_wires(fragment);
        
        // Add wires in deterministic order
        let mut sorted_wires: Vec<_> = needed_wires.into_iter().collect();
        sorted_wires.sort_by_key(|w| w.0);
        
        for wire_id in sorted_wires {
            let local_index = values.len();
            let value = full_witness.get(&wire_id).copied().unwrap_or(0);
            
            values.push(value);
            wire_to_index.insert(wire_id, local_index);
            
            // Check if this is a boundary wire
            if let Some(route) = self.routing.get(wire_id) {
                if route.is_boundary {
                    if route.producer_fragment == Some(fragment.id) {
                        // This fragment produces this boundary
                        boundary_outputs.push((wire_id, local_index));
                    }
                    if route.consumer_fragments.contains(&fragment.id) {
                        // This fragment consumes this boundary
                        boundary_inputs.push((wire_id, local_index));
                    }
                }
            }
        }
        
        FragmentWitness {
            fragment_id: fragment.id,
            values,
            wire_to_index,
            boundary_inputs,
            boundary_outputs,
        }
    }
    
    /// Collect all wires needed by a fragment
    fn collect_fragment_wires(&self, fragment: &FragmentSpec) -> HashSet<WireId> {
        let mut wires = HashSet::new();
        
        for &constraint_id in &fragment.constraints {
            if let Some(constraint) = self.graph.constraints.get(&constraint_id) {
                for &wire_id in &constraint.input_wires {
                    wires.insert(wire_id);
                }
                for &wire_id in &constraint.output_wires {
                    wires.insert(wire_id);
                }
            }
        }
        
        // Also include boundary wires
        for boundary in &fragment.input_boundaries {
            wires.insert(boundary.wire_id);
        }
        for boundary in &fragment.output_boundaries {
            wires.insert(boundary.wire_id);
        }
        
        wires
    }
}

/// Verify that fragment witnesses are consistent at boundaries
pub fn verify_boundary_consistency(
    witnesses: &[FragmentWitness],
    fragments: &[FragmentSpec],
) -> Result<(), String> {
    let witness_map: HashMap<_, _> = witnesses.iter()
        .map(|w| (w.fragment_id, w))
        .collect();
    
    for fragment in fragments {
        let witness = match witness_map.get(&fragment.id) {
            Some(w) => w,
            None => return Err(format!("Missing witness for fragment {}", fragment.id)),
        };
        
        // Check each input boundary
        for boundary in &fragment.input_boundaries {
            // Get value from this fragment
            let local_value = witness.wire_to_index.get(&boundary.wire_id)
                .and_then(|&idx| witness.values.get(idx))
                .copied();
            
            // Get value from source fragment
            let source_witness = match witness_map.get(&boundary.source_fragment) {
                Some(w) => w,
                None => return Err(format!(
                    "Missing witness for source fragment {}",
                    boundary.source_fragment
                )),
            };
            
            let source_value = source_witness.wire_to_index.get(&boundary.wire_id)
                .and_then(|&idx| source_witness.values.get(idx))
                .copied();
            
            if local_value != source_value {
                return Err(format!(
                    "Boundary mismatch for wire {} between {} and {}: {:?} vs {:?}",
                    boundary.wire_id,
                    boundary.source_fragment,
                    fragment.id,
                    source_value,
                    local_value
                ));
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fragment::BoundaryWire;

    #[test]
    fn test_witness_partitioning() {
        let mut graph = ConstraintGraph::new();
        
        // Simple setup
        use zk_fragment_graph::{Wire, Constraint, ConstraintType};
        
        graph.add_wire(Wire::internal(WireId(0)));
        graph.add_wire(Wire::internal(WireId(1)));
        
        let c0 = Constraint::new(ConstraintId(0), ConstraintType::Add, vec![], vec![WireId(0)]);
        let c1 = Constraint::new(ConstraintId(1), ConstraintType::Mul, vec![WireId(0)], vec![WireId(1)]);
        
        graph.add_constraint(c0);
        graph.add_constraint(c1);
        graph.build_edges_from_wires();
        
        let boundary = BoundaryWire {
            wire_id: WireId(0),
            source_fragment: FragmentId(0),
            target_fragment: FragmentId(1),
            source_index: 0,
            target_index: 0,
        };
        
        let mut f0 = FragmentSpec::new(FragmentId(0), vec![ConstraintId(0)]);
        f0.output_boundaries.push(boundary.clone());
        
        let mut f1 = FragmentSpec::new(FragmentId(1), vec![ConstraintId(1)]);
        f1.input_boundaries.push(boundary.clone());
        
        let fragments = vec![f0, f1];
        let routing = WireRoutingTable::from_boundaries(&[boundary]);
        
        let partitioner = WitnessPartitioner::new(graph, fragments.clone(), routing);
        
        let mut full_witness = HashMap::new();
        full_witness.insert(WireId(0), 42);
        full_witness.insert(WireId(1), 84);
        
        let partitioned = partitioner.partition(&full_witness);
        
        assert_eq!(partitioned.len(), 2);
        
        // Verify consistency
        assert!(verify_boundary_consistency(&partitioned, &fragments).is_ok());
    }
}

use zk_fragment_graph::ConstraintGraph;