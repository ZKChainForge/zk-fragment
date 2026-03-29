use super::{FragmentationStrategy, FragmentationConfig, FragmentationError, build_fragments_from_partition};
use crate::fragment::FragmentationResult;
use zk_fragment_graph::{ConstraintGraph, ConstraintId, algorithms};
use std::collections::HashMap;

/// Balanced K-way partitioning strategy
/// 
/// Aims to create fragments of roughly equal size while minimizing
/// the number of boundary wires.
pub struct BalancedStrategy;

impl FragmentationStrategy for BalancedStrategy {
    fn name(&self) -> &'static str {
        "Balanced"
    }
    
    fn fragment(
        &self,
        graph: &ConstraintGraph,
        config: &FragmentationConfig,
    ) -> Result<FragmentationResult, FragmentationError> {
        let constraint_count = graph.constraints.len();
        
        if constraint_count < config.min_fragment_size * 2 {
            return Err(FragmentationError::TooSmall(config.min_fragment_size * 2));
        }
        
        let target_fragments = config.target_fragment_count.unwrap_or_else(|| {
            (constraint_count / 500).max(2).min(8)
        });
        
        // Use depth-based partitioning for initial assignment
        let partition = self.depth_based_partition(graph, target_fragments)?;
        
        // Refine partition to balance sizes
        let refined = self.refine_partition(graph, partition, target_fragments, config)?;
        
        build_fragments_from_partition(graph, &refined, target_fragments)
    }
}

impl BalancedStrategy {
    pub fn new() -> Self {
        Self
    }
    
    /// Initial partition based on depth levels
    fn depth_based_partition(
        &self,
        graph: &ConstraintGraph,
        target_fragments: usize,
    ) -> Result<HashMap<ConstraintId, usize>, FragmentationError> {
        let levels = algorithms::group_by_depth(graph)
            .map_err(|e| FragmentationError::GraphValidation(e.to_string()))?;
        
        let total_constraints: usize = levels.iter().map(|l| l.len()).sum();
        let target_size = (total_constraints + target_fragments - 1) / target_fragments;
        
        let mut partition: HashMap<ConstraintId, usize> = HashMap::new();
        let mut current_fragment = 0;
        let mut current_size = 0;
        
        for level in levels {
            for id in level {
                partition.insert(id, current_fragment);
                current_size += 1;
                
                if current_size >= target_size && current_fragment < target_fragments - 1 {
                    current_fragment += 1;
                    current_size = 0;
                }
            }
        }
        
        Ok(partition)
    }
    
    /// Refine partition to improve balance
    fn refine_partition(
        &self,
        graph: &ConstraintGraph,
        mut partition: HashMap<ConstraintId, usize>,
        target_fragments: usize,
        config: &FragmentationConfig,
    ) -> Result<HashMap<ConstraintId, usize>, FragmentationError> {
        // Count sizes
        let mut sizes: Vec<usize> = vec![0; target_fragments];
        for &p in partition.values() {
            if p < target_fragments {
                sizes[p] += 1;
            }
        }
        
        let target_size = graph.constraints.len() / target_fragments;
        let max_size = config.max_fragment_size.unwrap_or(target_size * 2);
        let min_size = config.min_fragment_size;
        
        // Iteratively move constraints to balance
        for _ in 0..10 {
            let mut improved = false;
            
            // Find most imbalanced fragments
            let max_idx = sizes.iter().enumerate().max_by_key(|(_, &s)| s).map(|(i, _)| i);
            let min_idx = sizes.iter().enumerate()
                .filter(|(_, &s)| s > 0)
                .min_by_key(|(_, &s)| s)
                .map(|(i, _)| i);
            
            if let (Some(from), Some(to)) = (max_idx, min_idx) {
                if from != to && sizes[from] > min_size + 1 && sizes[to] < max_size {
                    // Find a constraint in 'from' that can move to 'to'
                    let moveable: Vec<_> = partition.iter()
                        .filter(|(_, &p)| p == from)
                        .map(|(&id, _)| id)
                        .collect();
                    
                    // Pick the one with most connections to 'to'
                    let best_move = moveable.iter()
                        .max_by_key(|&&id| {
                            self.count_connections_to_partition(graph, id, to, &partition)
                        });
                    
                    if let Some(&id) = best_move {
                        partition.insert(id, to);
                        sizes[from] -= 1;
                        sizes[to] += 1;
                        improved = true;
                    }
                }
            }
            
            if !improved {
                break;
            }
        }
        
        Ok(partition)
    }
    
    /// Count connections from a constraint to constraints in a partition
    fn count_connections_to_partition(
        &self,
        graph: &ConstraintGraph,
        constraint: ConstraintId,
        target_partition: usize,
        partition: &HashMap<ConstraintId, usize>,
    ) -> usize {
        let mut count = 0;
        
        for &succ in graph.get_successors(constraint) {
            if partition.get(&succ) == Some(&target_partition) {
                count += 1;
            }
        }
        
        for &pred in graph.get_predecessors(constraint) {
            if partition.get(&pred) == Some(&target_partition) {
                count += 1;
            }
        }
        
        count
    }
}

impl Default for BalancedStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zk_fragment_graph::builder::create_chain_circuit;

    #[test]
    fn test_balanced_strategy() {
        let graph = create_chain_circuit(100);
        let config = FragmentationConfig::with_fragment_count(4);
        
        let strategy = BalancedStrategy::new();
        let result = strategy.fragment(&graph, &config).unwrap();
        
        // Check roughly balanced
        let sizes: Vec<_> = result.fragments.iter()
            .map(|f| f.constraint_count())
            .collect();
        
        let max_size = *sizes.iter().max().unwrap();
        let min_size = *sizes.iter().min().unwrap();
        
        // Should be within 2x of each other
        assert!(max_size <= min_size * 3, "Sizes too imbalanced: {:?}", sizes);
    }
}