use crate::fragment::{FragmentId, BoundaryWire};
use zk_fragment_graph::WireId;
use std::collections::HashMap;

/// Wire routing table for tracking wire values across fragments
#[derive(Debug, Clone, Default)]
pub struct WireRoutingTable {
    /// Map from wire to its routing info
    routes: HashMap<WireId, WireRoute>,
}

/// Routing information for a single wire
#[derive(Debug, Clone)]
pub struct WireRoute {
    /// Wire ID
    pub wire_id: WireId,
    /// Fragment that produces this wire
    pub producer_fragment: Option<FragmentId>,
    /// Fragments that consume this wire
    pub consumer_fragments: Vec<FragmentId>,
    /// Is this wire a boundary?
    pub is_boundary: bool,
    /// Index in the global witness
    pub global_witness_index: Option<usize>,
    /// Index in each fragment's local witness
    pub fragment_witness_indices: HashMap<FragmentId, usize>,
}

impl WireRoutingTable {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }
    
    /// Build routing table from boundaries
    pub fn from_boundaries(boundaries: &[BoundaryWire]) -> Self {
        let mut table = Self::new();
        
        for boundary in boundaries {
            let route = table.routes.entry(boundary.wire_id).or_insert_with(|| {
                WireRoute {
                    wire_id: boundary.wire_id,
                    producer_fragment: Some(boundary.source_fragment),
                    consumer_fragments: Vec::new(),
                    is_boundary: true,
                    global_witness_index: None,
                    fragment_witness_indices: HashMap::new(),
                }
            });
            
            if !route.consumer_fragments.contains(&boundary.target_fragment) {
                route.consumer_fragments.push(boundary.target_fragment);
            }
        }
        
        table
    }
    
    /// Get route for a wire
    pub fn get(&self, wire_id: WireId) -> Option<&WireRoute> {
        self.routes.get(&wire_id)
    }
    
    /// Add or update a route
    pub fn add_route(&mut self, route: WireRoute) {
        self.routes.insert(route.wire_id, route);
    }
    
    /// Get all boundary wires
    pub fn get_boundaries(&self) -> Vec<WireId> {
        self.routes.iter()
            .filter(|(_, r)| r.is_boundary)
            .map(|(&id, _)| id)
            .collect()
    }
    
    /// Get wires produced by a fragment
    pub fn get_produced_by(&self, fragment: FragmentId) -> Vec<WireId> {
        self.routes.iter()
            .filter(|(_, r)| r.producer_fragment == Some(fragment))
            .map(|(&id, _)| id)
            .collect()
    }
    
    /// Get wires consumed by a fragment
    pub fn get_consumed_by(&self, fragment: FragmentId) -> Vec<WireId> {
        self.routes.iter()
            .filter(|(_, r)| r.consumer_fragments.contains(&fragment))
            .map(|(&id, _)| id)
            .collect()
    }
}

/// Witness partitioning information
#[derive(Debug, Clone)]
pub struct WitnessPartition {
    /// Fragment ID
    pub fragment_id: FragmentId,
    /// Global witness indices needed by this fragment
    pub required_indices: Vec<usize>,
    /// Mapping from global index to local index
    pub global_to_local: HashMap<usize, usize>,
    /// Boundary input indices (with commitment info)
    pub boundary_inputs: Vec<BoundaryInputInfo>,
    /// Boundary output indices
    pub boundary_outputs: Vec<BoundaryOutputInfo>,
}

#[derive(Debug, Clone)]
pub struct BoundaryInputInfo {
    pub wire_id: WireId,
    pub local_index: usize,
    pub source_fragment: FragmentId,
}

#[derive(Debug, Clone)]
pub struct BoundaryOutputInfo {
    pub wire_id: WireId,
    pub local_index: usize,
    pub target_fragments: Vec<FragmentId>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wire_routing_table() {
        let boundaries = vec![
            BoundaryWire {
                wire_id: WireId(0),
                source_fragment: FragmentId(0),
                target_fragment: FragmentId(1),
                source_index: 0,
                target_index: 0,
            },
        ];
        
        let table = WireRoutingTable::from_boundaries(&boundaries);
        
        let route = table.get(WireId(0)).unwrap();
        assert!(route.is_boundary);
        assert_eq!(route.producer_fragment, Some(FragmentId(0)));
        assert!(route.consumer_fragments.contains(&FragmentId(1)));
    }
}