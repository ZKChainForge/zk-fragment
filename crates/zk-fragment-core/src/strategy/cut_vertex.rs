use super::{FragmentationStrategy, FragmentationConfig, FragmentationError, build_fragments_from_partition};
use crate::fragment::FragmentationResult;
use zk_fragment_graph::{ConstraintGraph, ConstraintId, algorithms};
use std::collections::{HashMap, HashSet, VecDeque};

/// Fragmentation strategy based on cut vertices
/// 
/// Cut vertices are natural fragmentation points because removing them
/// disconnects the graph, meaning they have minimal boundary requirements.
pub struct CutVertexStrategy;

impl FragmentationStrategy for CutVertexStrategy {
    fn name(&self) -> &'static str {
        "CutVertex"
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
        
        // Find cut vertices
        let cut_analysis = algorithms::find_cut_vertices(graph);
        
        if cut_analysis.cut_vertices.is_empty() {
            // No natural cut points, fall back to simple bisection
            return self.bisect_graph(graph, config);
        }
        
        // Score and select cut vertices
        let scores = algorithms::score_cut_candidates(graph);
        let mut scored_cuts: Vec<_> = cut_analysis.cut_vertices.iter()
            .map(|&id| (id, scores.get(&id).copied().unwrap_or(0.0)))
            .collect();
        scored_cuts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Determine how many cuts to make
        let target_fragments = config.target_fragment_count.unwrap_or_else(|| {
            // Auto-determine based on graph size
            let suggested = (constraint_count / 1000).max(2).min(8);
            suggested
        });
        
        let num_cuts = (target_fragments - 1).min(scored_cuts.len());
        
        if num_cuts == 0 {
            return self.bisect_graph(graph, config);
        }
        
        // Select best cut vertices
        let selected_cuts: Vec<ConstraintId> = scored_cuts.iter()
            .take(num_cuts)
            .map(|(id, _)| *id)
            .collect();
        
        // Create partition by removing cut vertices and finding components
        let partition = self.partition_by_cuts(graph, &selected_cuts)?;
        
        build_fragments_from_partition(graph, &partition, target_fragments)
    }
}

impl CutVertexStrategy {
    pub fn new() -> Self {
        Self
    }
    
    /// Partition graph by treating cut vertices as boundaries
    fn partition_by_cuts(
        &self,
        graph: &ConstraintGraph,
        cut_vertices: &[ConstraintId],
    ) -> Result<HashMap<ConstraintId, usize>, FragmentationError> {
        let cut_set: HashSet<_> = cut_vertices.iter().copied().collect();
        let mut partition: HashMap<ConstraintId, usize> = HashMap::new();
        let mut visited: HashSet<ConstraintId> = HashSet::new();
        let mut current_partition = 0;
        
        // Build undirected adjacency for component finding
        let mut adj: HashMap<ConstraintId, Vec<ConstraintId>> = HashMap::new();
        for &id in graph.constraints.keys() {
            adj.insert(id, Vec::new());
        }
        for &id in graph.constraints.keys() {
            for &succ in graph.get_successors(id) {
                adj.get_mut(&id).unwrap().push(succ);
                adj.get_mut(&succ).unwrap().push(id);
            }
        }
        
        // BFS to find connected components when cut vertices are "removed"
        for &start in graph.constraints.keys() {
            if visited.contains(&start) || cut_set.contains(&start) {
                continue;
            }
            
            // BFS from this node, not crossing cut vertices
            let mut queue = VecDeque::new();
            queue.push_back(start);
            visited.insert(start);
            partition.insert(start, current_partition);
            
            while let Some(node) = queue.pop_front() {
                if let Some(neighbors) = adj.get(&node) {
                    for &neighbor in neighbors {
                        if !visited.contains(&neighbor) && !cut_set.contains(&neighbor) {
                            visited.insert(neighbor);
                            partition.insert(neighbor, current_partition);
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
            
            current_partition += 1;
        }
        
        // Assign cut vertices to appropriate partitions
        // (assign to the partition of their first successor, or create new ones)
        for &cut in cut_vertices {
            // Find which partition most of its neighbors belong to
            let mut neighbor_partitions: HashMap<usize, usize> = HashMap::new();
            
            if let Some(neighbors) = adj.get(&cut) {
                for &neighbor in neighbors {
                    if let Some(&p) = partition.get(&neighbor) {
                        *neighbor_partitions.entry(p).or_insert(0) += 1;
                    }
                }
            }
            
            // Assign to most common neighbor partition
            let assigned = neighbor_partitions.iter()
                .max_by_key(|(_, &count)| count)
                .map(|(&p, _)| p)
                .unwrap_or(current_partition);
            
            partition.insert(cut, assigned);
        }
        
        Ok(partition)
    }
    
    /// Simple bisection when no cut vertices are available
    fn bisect_graph(
        &self,
        graph: &ConstraintGraph,
        config: &FragmentationConfig,
    ) -> Result<FragmentationResult, FragmentationError> {
        let target_fragments = config.target_fragment_count.unwrap_or(2);
        
        // Use topological order to partition
        let order = algorithms::topological_sort(graph)
            .map_err(|e| FragmentationError::GraphValidation(e.to_string()))?;
        
        let fragment_size = (order.len() + target_fragments - 1) / target_fragments;
        
        let mut partition: HashMap<ConstraintId, usize> = HashMap::new();
        for (i, id) in order.iter().enumerate() {
            let fragment_id = i / fragment_size;
            partition.insert(*id, fragment_id.min(target_fragments - 1));
        }
        
        build_fragments_from_partition(graph, &partition, target_fragments)
    }
}

impl Default for CutVertexStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zk_fragment_graph::builder::{GraphBuilder, create_chain_circuit, create_diamond_circuit};

    #[test]
    fn test_cut_vertex_chain() {
        let graph = create_chain_circuit(20);
        let config = FragmentationConfig::with_fragment_count(4);
        
        let strategy = CutVertexStrategy::new();
        let result = strategy.fragment(&graph, &config);
        
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.fragments.is_empty());
    }

    #[test]
    fn test_cut_vertex_diamond() {
        let graph = create_diamond_circuit(4, 3);
        let config = FragmentationConfig::with_fragment_count(2);
        
        let strategy = CutVertexStrategy::new();
        let result = strategy.fragment(&graph, &config);
        
        assert!(result.is_ok());
    }
}