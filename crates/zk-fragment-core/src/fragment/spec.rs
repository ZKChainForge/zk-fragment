use serde::{Deserialize, Serialize};
use zk_fragment_graph::{ConstraintId, WireId};
use std::collections::HashSet;

/// Unique identifier for a fragment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FragmentId(pub usize);

impl std::fmt::Display for FragmentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "F{}", self.0)
    }
}

/// A boundary wire specification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BoundaryWire {
    /// The wire that crosses fragment boundaries
    pub wire_id: WireId,
    /// Fragment that produces this wire's value
    pub source_fragment: FragmentId,
    /// Fragment that consumes this wire's value
    pub target_fragment: FragmentId,
    /// Index in source fragment's output boundaries
    pub source_index: usize,
    /// Index in target fragment's input boundaries
    pub target_index: usize,
}

/// Specification for a single fragment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentSpec {
    /// Unique identifier
    pub id: FragmentId,
    /// Constraints belonging to this fragment
    pub constraints: Vec<ConstraintId>,
    /// Wires that enter this fragment from other fragments
    pub input_boundaries: Vec<BoundaryWire>,
    /// Wires that exit this fragment to other fragments
    pub output_boundaries: Vec<BoundaryWire>,
    /// Public input wires used by this fragment
    pub public_inputs: Vec<WireId>,
    /// Public output wires produced by this fragment
    pub public_outputs: Vec<WireId>,
    /// Internal wires (produced and consumed within this fragment)
    pub internal_wires: Vec<WireId>,
    /// Fragments this fragment depends on (must be proven first)
    pub dependencies: Vec<FragmentId>,
    /// Fragments that depend on this fragment
    pub dependents: Vec<FragmentId>,
}

impl FragmentSpec {
    pub fn new(id: FragmentId, constraints: Vec<ConstraintId>) -> Self {
        Self {
            id,
            constraints,
            input_boundaries: Vec::new(),
            output_boundaries: Vec::new(),
            public_inputs: Vec::new(),
            public_outputs: Vec::new(),
            internal_wires: Vec::new(),
            dependencies: Vec::new(),
            dependents: Vec::new(),
        }
    }
    
    /// Number of constraints in this fragment
    pub fn constraint_count(&self) -> usize {
        self.constraints.len()
    }
    
    /// Total number of boundary wires
    pub fn boundary_count(&self) -> usize {
        self.input_boundaries.len() + self.output_boundaries.len()
    }
    
    /// Check if this fragment has any dependencies
    pub fn has_dependencies(&self) -> bool {
        !self.dependencies.is_empty()
    }
    
    /// Check if this fragment can start proving (all dependencies met)
    pub fn can_start(&self, completed: &HashSet<FragmentId>) -> bool {
        self.dependencies.iter().all(|dep| completed.contains(dep))
    }
}

/// Result of fragmenting a circuit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentationResult {
    /// All fragments
    pub fragments: Vec<FragmentSpec>,
    /// Total number of boundary wires
    pub total_boundary_wires: usize,
    /// Maximum depth of fragment dependency chain
    pub dependency_depth: usize,
    /// Maximum fragments that can be proven in parallel
    pub max_parallelism: usize,
    /// Fragment execution order (topological order)
    pub execution_order: Vec<FragmentId>,
}

impl FragmentationResult {
    /// Get fragment by ID
    pub fn get_fragment(&self, id: FragmentId) -> Option<&FragmentSpec> {
        self.fragments.iter().find(|f| f.id == id)
    }
    
    /// Get all fragments at a given dependency level
    pub fn get_level(&self, level: usize) -> Vec<&FragmentSpec> {
        // Group by dependency depth
        let mut current_level = 0;
        let mut current_set: HashSet<FragmentId> = HashSet::new();
        
        // Find fragments with no dependencies (level 0)
        let mut result = Vec::new();
        
        for fragment in &self.fragments {
            let frag_level = self.compute_fragment_level(fragment.id);
            if frag_level == level {
                result.push(fragment);
            }
        }
        
        result
    }
    
    fn compute_fragment_level(&self, id: FragmentId) -> usize {
        let fragment = match self.get_fragment(id) {
            Some(f) => f,
            None => return 0,
        };
        
        if fragment.dependencies.is_empty() {
            return 0;
        }
        
        fragment.dependencies.iter()
            .map(|&dep| self.compute_fragment_level(dep) + 1)
            .max()
            .unwrap_or(0)
    }
    
    /// Get boundary connections as (source_fragment, target_fragment, wire) tuples
    pub fn get_boundary_connections(&self) -> Vec<(FragmentId, FragmentId, WireId)> {
        let mut connections = Vec::new();
        
        for fragment in &self.fragments {
            for boundary in &fragment.output_boundaries {
                connections.push((
                    boundary.source_fragment,
                    boundary.target_fragment,
                    boundary.wire_id,
                ));
            }
        }
        
        connections
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fragment_spec_creation() {
        let fragment = FragmentSpec::new(
            FragmentId(0),
            vec![ConstraintId(0), ConstraintId(1), ConstraintId(2)],
        );
        
        assert_eq!(fragment.constraint_count(), 3);
        assert_eq!(fragment.boundary_count(), 0);
        assert!(!fragment.has_dependencies());
    }

    #[test]
    fn test_fragment_can_start() {
        let mut fragment = FragmentSpec::new(FragmentId(1), vec![ConstraintId(0)]);
        fragment.dependencies = vec![FragmentId(0)];
        
        let mut completed = HashSet::new();
        assert!(!fragment.can_start(&completed));
        
        completed.insert(FragmentId(0));
        assert!(fragment.can_start(&completed));
    }
}