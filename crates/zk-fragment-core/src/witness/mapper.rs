use crate::fragment::FragmentId;
use zk_fragment_graph::WireId;
use std::collections::HashMap;

/// Maps global witness indices to fragment-local indices
#[derive(Debug, Clone, Default)]
pub struct WitnessMapper {
    /// For each fragment, maps wire ID to local index
    fragment_mappings: HashMap<FragmentId, HashMap<WireId, usize>>,
    /// Maps wire ID to global index
    global_mapping: HashMap<WireId, usize>,
}

impl WitnessMapper {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a global wire mapping
    pub fn add_global(&mut self, wire_id: WireId, index: usize) {
        self.global_mapping.insert(wire_id, index);
    }
    
    /// Add a fragment-local wire mapping
    pub fn add_local(&mut self, fragment: FragmentId, wire_id: WireId, index: usize) {
        self.fragment_mappings
            .entry(fragment)
            .or_default()
            .insert(wire_id, index);
    }
    
    /// Get global index for a wire
    pub fn get_global(&self, wire_id: WireId) -> Option<usize> {
        self.global_mapping.get(&wire_id).copied()
    }
    
    /// Get local index for a wire in a fragment
    pub fn get_local(&self, fragment: FragmentId, wire_id: WireId) -> Option<usize> {
        self.fragment_mappings
            .get(&fragment)
            .and_then(|m| m.get(&wire_id))
            .copied()
    }
    
    /// Get all local mappings for a fragment
    pub fn get_fragment_mapping(&self, fragment: FragmentId) -> Option<&HashMap<WireId, usize>> {
        self.fragment_mappings.get(&fragment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_witness_mapper() {
        let mut mapper = WitnessMapper::new();
        
        mapper.add_global(WireId(0), 0);
        mapper.add_global(WireId(1), 1);
        
        mapper.add_local(FragmentId(0), WireId(0), 0);
        mapper.add_local(FragmentId(1), WireId(0), 0);
        mapper.add_local(FragmentId(1), WireId(1), 1);
        
        assert_eq!(mapper.get_global(WireId(0)), Some(0));
        assert_eq!(mapper.get_local(FragmentId(0), WireId(0)), Some(0));
        assert_eq!(mapper.get_local(FragmentId(1), WireId(1)), Some(1));
    }
}