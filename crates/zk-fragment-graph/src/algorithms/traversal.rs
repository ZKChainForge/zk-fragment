use crate::types::{ConstraintGraph, ConstraintId};
use std::collections::{HashSet, VecDeque};

/// Breadth-first search from a starting constraint
/// Returns constraints in BFS order
pub fn bfs(graph: &ConstraintGraph, start: ConstraintId) -> Vec<ConstraintId> {
    let mut visited: HashSet<ConstraintId> = HashSet::new();
    let mut queue: VecDeque<ConstraintId> = VecDeque::new();
    let mut result: Vec<ConstraintId> = Vec::new();
    
    queue.push_back(start);
    visited.insert(start);
    
    while let Some(node) = queue.pop_front() {
        result.push(node);
        
        for &succ in graph.get_successors(node) {
            if !visited.contains(&succ) {
                visited.insert(succ);
                queue.push_back(succ);
            }
        }
    }
    
    result
}

/// Depth-first search from a starting constraint
/// Returns constraints in DFS order
pub fn dfs(graph: &ConstraintGraph, start: ConstraintId) -> Vec<ConstraintId> {
    let mut visited: HashSet<ConstraintId> = HashSet::new();
    let mut result: Vec<ConstraintId> = Vec::new();
    
    dfs_recursive(graph, start, &mut visited, &mut result);
    
    result
}

fn dfs_recursive(
    graph: &ConstraintGraph,
    node: ConstraintId,
    visited: &mut HashSet<ConstraintId>,
    result: &mut Vec<ConstraintId>,
) {
    if visited.contains(&node) {
        return;
    }
    
    visited.insert(node);
    result.push(node);
    
    for &succ in graph.get_successors(node) {
        dfs_recursive(graph, succ, visited, result);
    }
}

/// Find all constraints reachable from a starting set
pub fn reachable_from(graph: &ConstraintGraph, starts: &[ConstraintId]) -> HashSet<ConstraintId> {
    let mut reachable: HashSet<ConstraintId> = HashSet::new();
    let mut queue: VecDeque<ConstraintId> = VecDeque::new();
    
    for &start in starts {
        if !reachable.contains(&start) {
            queue.push_back(start);
            reachable.insert(start);
        }
    }
    
    while let Some(node) = queue.pop_front() {
        for &succ in graph.get_successors(node) {
            if !reachable.contains(&succ) {
                reachable.insert(succ);
                queue.push_back(succ);
            }
        }
    }
    
    reachable
}

/// Find all constraints that can reach a target set
pub fn reaching_to(graph: &ConstraintGraph, targets: &[ConstraintId]) -> HashSet<ConstraintId> {
    let mut reaching: HashSet<ConstraintId> = HashSet::new();
    let mut queue: VecDeque<ConstraintId> = VecDeque::new();
    
    for &target in targets {
        if !reaching.contains(&target) {
            queue.push_back(target);
            reaching.insert(target);
        }
    }
    
    while let Some(node) = queue.pop_front() {
        for &pred in graph.get_predecessors(node) {
            if !reaching.contains(&pred) {
                reaching.insert(pred);
                queue.push_back(pred);
            }
        }
    }
    
    reaching
}

/// Check if there's a path from source to target
pub fn has_path(graph: &ConstraintGraph, source: ConstraintId, target: ConstraintId) -> bool {
    if source == target {
        return true;
    }
    
    let reachable = reachable_from(graph, &[source]);
    reachable.contains(&target)
}

/// Find shortest path from source to target
/// Returns None if no path exists
pub fn shortest_path(
    graph: &ConstraintGraph,
    source: ConstraintId,
    target: ConstraintId,
) -> Option<Vec<ConstraintId>> {
    if source == target {
        return Some(vec![source]);
    }
    
    let mut visited: HashSet<ConstraintId> = HashSet::new();
    let mut queue: VecDeque<ConstraintId> = VecDeque::new();
    let mut parent: std::collections::HashMap<ConstraintId, ConstraintId> = std::collections::HashMap::new();
    
    queue.push_back(source);
    visited.insert(source);
    
    while let Some(node) = queue.pop_front() {
        for &succ in graph.get_successors(node) {
            if !visited.contains(&succ) {
                visited.insert(succ);
                parent.insert(succ, node);
                
                if succ == target {
                    // Reconstruct path
                    let mut path = vec![target];
                    let mut current = target;
                    while let Some(&p) = parent.get(&current) {
                        path.push(p);
                        current = p;
                    }
                    path.reverse();
                    return Some(path);
                }
                
                queue.push_back(succ);
            }
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn create_test_graph() -> ConstraintGraph {
        // A -> B -> C
        //      |
        //      v
        //      D
        let mut graph = ConstraintGraph::new();
        
        for i in 0..4 {
            graph.add_wire(Wire::internal(WireId(i)));
        }
        
        let c_a = Constraint::new(ConstraintId(0), ConstraintType::Add, vec![], vec![WireId(0)]);
        let c_b = Constraint::new(ConstraintId(1), ConstraintType::Mul, vec![WireId(0)], vec![WireId(1), WireId(2)]);
        let c_c = Constraint::new(ConstraintId(2), ConstraintType::Add, vec![WireId(1)], vec![]);
        let c_d = Constraint::new(ConstraintId(3), ConstraintType::Add, vec![WireId(2)], vec![]);
        
        graph.add_constraint(c_a);
        graph.add_constraint(c_b);
        graph.add_constraint(c_c);
        graph.add_constraint(c_d);
        
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
    fn test_bfs() {
        let graph = create_test_graph();
        let order = bfs(&graph, ConstraintId(0));
        
        assert_eq!(order[0], ConstraintId(0));
        assert_eq!(order.len(), 4);
    }

    #[test]
    fn test_dfs() {
        let graph = create_test_graph();
        let order = dfs(&graph, ConstraintId(0));
        
        assert_eq!(order[0], ConstraintId(0));
        assert_eq!(order.len(), 4);
    }

    #[test]
    fn test_has_path() {
        let graph = create_test_graph();
        
        assert!(has_path(&graph, ConstraintId(0), ConstraintId(2)));
        assert!(has_path(&graph, ConstraintId(0), ConstraintId(3)));
        assert!(!has_path(&graph, ConstraintId(2), ConstraintId(0)));
    }

    #[test]
    fn test_shortest_path() {
        let graph = create_test_graph();
        
        let path = shortest_path(&graph, ConstraintId(0), ConstraintId(2)).unwrap();
        assert_eq!(path, vec![ConstraintId(0), ConstraintId(1), ConstraintId(2)]);
    }

    #[test]
    fn test_reachable_from() {
        let graph = create_test_graph();
        
        let reachable = reachable_from(&graph, &[ConstraintId(1)]);
        assert!(reachable.contains(&ConstraintId(1)));
        assert!(reachable.contains(&ConstraintId(2)));
        assert!(reachable.contains(&ConstraintId(3)));
        assert!(!reachable.contains(&ConstraintId(0)));
    }
}