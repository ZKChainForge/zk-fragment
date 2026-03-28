use crate::types::{ConstraintGraph, ConstraintId};
use std::collections::{HashMap, VecDeque};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TopologicalSortError {
    #[error("Graph contains a cycle involving constraint {0}")]
    CycleDetected(ConstraintId),
    #[error("Graph is empty")]
    EmptyGraph,
}

/// Perform topological sort using Kahn's algorithm
/// Returns constraints in valid execution order (dependencies before dependents)
pub fn topological_sort(graph: &ConstraintGraph) -> Result<Vec<ConstraintId>, TopologicalSortError> {
    if graph.constraints.is_empty() {
        return Err(TopologicalSortError::EmptyGraph);
    }
    
    // Compute in-degrees
    let mut in_degree: HashMap<ConstraintId, usize> = HashMap::new();
    for &id in graph.constraints.keys() {
        in_degree.insert(id, graph.in_degree(id));
    }
    
    // Initialize queue with nodes that have no incoming edges
    let mut queue: VecDeque<ConstraintId> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&id, _)| id)
        .collect();
    
    let mut result = Vec::with_capacity(graph.constraints.len());
    
    while let Some(node) = queue.pop_front() {
        result.push(node);
        
        // For each successor, decrease in-degree
        for &successor in graph.get_successors(node) {
            if let Some(deg) = in_degree.get_mut(&successor) {
                *deg -= 1;
                if *deg == 0 {
                    queue.push_back(successor);
                }
            }
        }
    }
    
    // If we didn't process all nodes, there's a cycle
    if result.len() != graph.constraints.len() {
        // Find a node that's part of a cycle
        let cycle_node = in_degree
            .iter()
            .find(|(_, &deg)| deg > 0)
            .map(|(&id, _)| id)
            .unwrap();
        return Err(TopologicalSortError::CycleDetected(cycle_node));
    }
    
    Ok(result)
}

/// Compute depth/level of each constraint
/// Depth 0 = no dependencies, Depth N = longest path from any source
pub fn compute_depths(graph: &ConstraintGraph) -> Result<HashMap<ConstraintId, usize>, TopologicalSortError> {
    let order = topological_sort(graph)?;
    let mut depths: HashMap<ConstraintId, usize> = HashMap::new();
    
    // Initialize all depths to 0
    for &id in &order {
        depths.insert(id, 0);
    }
    
    // Process in topological order
    for id in order {
        let current_depth = depths[&id];
        for &successor in graph.get_successors(id) {
            let new_depth = current_depth + 1;
            if new_depth > depths[&successor] {
                depths.insert(successor, new_depth);
            }
        }
    }
    
    Ok(depths)
}

/// Group constraints by depth level
/// Returns Vec where index is depth and value is list of constraints at that depth
pub fn group_by_depth(graph: &ConstraintGraph) -> Result<Vec<Vec<ConstraintId>>, TopologicalSortError> {
    let depths = compute_depths(graph)?;
    
    let max_depth = depths.values().copied().max().unwrap_or(0);
    let mut levels: Vec<Vec<ConstraintId>> = vec![Vec::new(); max_depth + 1];
    
    for (id, depth) in depths {
        levels[depth].push(id);
    }
    
    Ok(levels)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn create_simple_dag() -> ConstraintGraph {
        // A -> B -> C
        //      |
        //      v
        //      D
        let mut graph = ConstraintGraph::new();
        
        // Add wires
        graph.add_wire(Wire::internal(WireId(0)));
        graph.add_wire(Wire::internal(WireId(1)));
        graph.add_wire(Wire::internal(WireId(2)));
        
        // Add constraints
        let c_a = Constraint::new(
            ConstraintId(0),
            ConstraintType::Add,
            vec![],
            vec![WireId(0)],
        );
        let c_b = Constraint::new(
            ConstraintId(1),
            ConstraintType::Mul,
            vec![WireId(0)],
            vec![WireId(1), WireId(2)],
        );
        let c_c = Constraint::new(
            ConstraintId(2),
            ConstraintType::Add,
            vec![WireId(1)],
            vec![],
        );
        let c_d = Constraint::new(
            ConstraintId(3),
            ConstraintType::Add,
            vec![WireId(2)],
            vec![],
        );
        
        graph.add_constraint(c_a);
        graph.add_constraint(c_b);
        graph.add_constraint(c_c);
        graph.add_constraint(c_d);
        
        // Update wire producers/consumers
        graph.wires.get_mut(&WireId(0)).unwrap().producer = Some(ConstraintId(0));
        graph.wires.get_mut(&WireId(0)).unwrap().consumers.push(ConstraintId(1));
        
        graph.wires.get_mut(&WireId(1)).unwrap().producer = Some(ConstraintId(1));
        graph.wires.get_mut(&WireId(1)).unwrap().consumers.push(ConstraintId(2));
        
        graph.wires.get_mut(&WireId(2)).unwrap().producer = Some(ConstraintId(1));
        graph.wires.get_mut(&WireId(2)).unwrap().consumers.push(ConstraintId(3));
        
        graph.build_edges_from_wires();
        
        graph
    }

    #[test]
    fn test_topological_sort_simple() {
        let graph = create_simple_dag();
        let order = topological_sort(&graph).unwrap();
        
        assert_eq!(order.len(), 4);
        
        // A must come before B
        let pos_a = order.iter().position(|&x| x == ConstraintId(0)).unwrap();
        let pos_b = order.iter().position(|&x| x == ConstraintId(1)).unwrap();
        assert!(pos_a < pos_b);
        
        // B must come before C and D
        let pos_c = order.iter().position(|&x| x == ConstraintId(2)).unwrap();
        let pos_d = order.iter().position(|&x| x == ConstraintId(3)).unwrap();
        assert!(pos_b < pos_c);
        assert!(pos_b < pos_d);
    }

    #[test]
    fn test_compute_depths() {
        let graph = create_simple_dag();
        let depths = compute_depths(&graph).unwrap();
        
        assert_eq!(depths[&ConstraintId(0)], 0); // A: no dependencies
        assert_eq!(depths[&ConstraintId(1)], 1); // B: depends on A
        assert_eq!(depths[&ConstraintId(2)], 2); // C: depends on B
        assert_eq!(depths[&ConstraintId(3)], 2); // D: depends on B
    }

    #[test]
    fn test_group_by_depth() {
        let graph = create_simple_dag();
        let levels = group_by_depth(&graph).unwrap();
        
        assert_eq!(levels.len(), 3);
        assert_eq!(levels[0], vec![ConstraintId(0)]);
        assert_eq!(levels[1], vec![ConstraintId(1)]);
        assert!(levels[2].contains(&ConstraintId(2)));
        assert!(levels[2].contains(&ConstraintId(3)));
    }
}