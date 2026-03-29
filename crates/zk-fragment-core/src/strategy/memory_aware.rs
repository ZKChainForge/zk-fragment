use super::{FragmentationStrategy, FragmentationConfig, FragmentationError, build_fragments_from_partition};
use crate::fragment::FragmentationResult;
use zk_fragment_graph::{ConstraintGraph, ConstraintId, algorithms};
use std::collections::HashMap;

/// Memory-aware fragmentation strategy
/// 
/// Creates fragments that respect memory limits, preventing OOM during proving.
pub struct MemoryAwareStrategy;

impl FragmentationStrategy for MemoryAwareStrategy {
    fn name(&self) -> &'static str {
        "MemoryAware"
    }
    
    fn fragment(
        &self,
        graph: &ConstraintGraph,
        config: &FragmentationConfig,
    ) -> Result<FragmentationResult, FragmentationError> {
        let constraint_count = graph.constraints.len();
        
        if constraint_count < config.min_fragment_size {
            return Err(FragmentationError::TooSmall(config.min_fragment_size));
        }
        
        let max_memory = config.max_memory_per_fragment.unwrap_or(8 * 1024 * 1024 * 1024); // 8GB default
        
        // Estimate memory per constraint (rough heuristic)
        let memory_per_constraint = self.estimate_memory_per_constraint(graph);
        let max_constraints_per_fragment = (max_memory / memory_per_constraint) as usize;
        let max_constraints_per_fragment = max_constraints_per_fragment
            .max(config.min_fragment_size)
            .min(config.max_fragment_size.unwrap_or(usize::MAX));
        
        // Get topological order
        let order = algorithms::topological_sort(graph)
            .map_err(|e| FragmentationError::GraphValidation(e.to_string()))?;
        
        // Greedily assign to fragments
        let mut partition: HashMap<ConstraintId, usize> = HashMap::new();
        let mut current_fragment = 0;
        let mut current_size = 0;
        let mut current_memory: u64 = 0;
        
        for id in order {
            let constraint_memory = self.estimate_constraint_memory(graph, id);
            
            // Check if we need to start a new fragment
            let would_exceed = current_memory + constraint_memory > max_memory;
            let would_exceed_size = current_size >= max_constraints_per_fragment;
            
            if (would_exceed || would_exceed_size) && current_size >= config.min_fragment_size {
                current_fragment += 1;
                current_size = 0;
                current_memory = 0;
            }
            
            partition.insert(id, current_fragment);
            current_size += 1;
            current_memory += constraint_memory;
        }
        
        let num_fragments = current_fragment + 1;
        build_fragments_from_partition(graph, &partition, num_fragments)
    }
}

impl MemoryAwareStrategy {
    pub fn new() -> Self {
        Self
    }
    
    /// Estimate memory per constraint (in bytes)
    fn estimate_memory_per_constraint(&self, graph: &ConstraintGraph) -> u64 {
        // Rough estimate based on typical ZK proving requirements
        // This should be calibrated based on actual profiling
        
        let avg_wires_per_constraint = if graph.constraints.is_empty() {
            2.0
        } else {
            graph.wires.len() as f64 / graph.constraints.len() as f64
        };
        
        // Base memory: ~5KB per constraint
        // Plus ~1KB per wire
        let base_memory = 5 * 1024;
        let wire_memory = (avg_wires_per_constraint * 1024.0) as u64;
        
        base_memory + wire_memory
    }
    
    /// Estimate memory for a specific constraint
    fn estimate_constraint_memory(&self, graph: &ConstraintGraph, id: ConstraintId) -> u64 {
        let constraint = match graph.constraints.get(&id) {
            Some(c) => c,
            None => return 5 * 1024, // Default
        };
        
        let base_memory: u64 = match &constraint.constraint_type {
            zk_fragment_graph::ConstraintType::Poseidon => 20 * 1024, // Hash is expensive
            zk_fragment_graph::ConstraintType::RangeCheck => 10 * 1024,
            zk_fragment_graph::ConstraintType::Mul => 8 * 1024,
            zk_fragment_graph::ConstraintType::Add => 5 * 1024,
            _ => 5 * 1024,
        };
        
        let wire_count = constraint.input_wires.len() + constraint.output_wires.len();
        let wire_memory = wire_count as u64 * 512;
        
        base_memory + wire_memory
    }
}

impl Default for MemoryAwareStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zk_fragment_graph::builder::create_chain_circuit;

    #[test]
    fn test_memory_aware_strategy() {
        let graph = create_chain_circuit(100);
        
        // Set a low memory limit to force multiple fragments
        let config = FragmentationConfig {
            strategy: super::super::StrategyType::MemoryAware,
            max_memory_per_fragment: Some(100 * 1024), // 100KB - very low
            min_fragment_size: 5,
            ..Default::default()
        };
        
        let strategy = MemoryAwareStrategy::new();
        let result = strategy.fragment(&graph, &config).unwrap();
        
        // Should create multiple fragments due to low memory limit
        assert!(result.fragments.len() > 1);
    }
}