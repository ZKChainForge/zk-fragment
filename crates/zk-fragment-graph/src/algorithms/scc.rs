use crate::types::{ConstraintGraph, ConstraintId};
use std::collections::{HashMap, HashSet};

/// Strongly Connected Component
#[derive(Debug, Clone)]
pub struct SCC {
    /// Component ID
    pub id: usize,
    /// Constraints in this component
    pub members: Vec<ConstraintId>,
}

/// Result of SCC analysis
#[derive(Debug, Clone)]
pub struct SCCAnalysis {
    /// All strongly connected components
    pub components: Vec<SCC>,
    /// Map from constraint to its component ID
    pub component_map: HashMap<ConstraintId, usize>,
    /// True if graph is a DAG (all SCCs have size 1)
    pub is_dag: bool,
}

/// Find strongly connected components using Kosaraju's algorithm
pub fn find_sccs(graph: &ConstraintGraph) -> SCCAnalysis {
    let constraint_ids: Vec<ConstraintId> = graph.constraints.keys().copied().collect();
    
    if constraint_ids.is_empty() {
        return SCCAnalysis {
            components: Vec::new(),
            component_map: HashMap::new(),
            is_dag: true,
        };
    }
    
    // First pass: DFS to get finish order
    let mut visited: HashSet<ConstraintId> = HashSet::new();
    let mut finish_order: Vec<ConstraintId> = Vec::new();
    
    for &id in &constraint_ids {
        if !visited.contains(&id) {
            dfs_first_pass(id, graph, &mut visited, &mut finish_order);
        }
    }
    
    // Build reverse graph
    let mut reverse_adj: HashMap<ConstraintId, Vec<ConstraintId>> = HashMap::new();
    for &id in &constraint_ids {
        reverse_adj.insert(id, Vec::new());
    }
    
    for &id in &constraint_ids {
        for &succ in graph.get_successors(id) {
            reverse_adj.get_mut(&succ).unwrap().push(id);
        }
    }
    
    // Second pass: DFS on reverse graph in reverse finish order
    visited.clear();
    let mut components: Vec<SCC> = Vec::new();
    let mut component_map: HashMap<ConstraintId, usize> = HashMap::new();
    
    for &id in finish_order.iter().rev() {
        if !visited.contains(&id) {
            let mut component_members: Vec<ConstraintId> = Vec::new();
            dfs_second_pass(id, &reverse_adj, &mut visited, &mut component_members);
            
            let component_id = components.len();
            for &member in &component_members {
                component_map.insert(member, component_id);
            }
            
            components.push(SCC {
                id: component_id,
                members: component_members,
            });
        }
    }
    
    // Check if DAG (all components have size 1)
    let is_dag = components.iter().all(|c| c.members.len() == 1);
    
    SCCAnalysis {
        components,
        component_map,
        is_dag,
    }
}

fn dfs_first_pass(
    node: ConstraintId,
    graph: &ConstraintGraph,
    visited: &mut HashSet<ConstraintId>,
    finish_order: &mut Vec<ConstraintId>,
) {
    visited.insert(node);
    
    for &succ in graph.get_successors(node) {
        if !visited.contains(&succ) {
            dfs_first_pass(succ, graph, visited, finish_order);
        }
    }
    
    finish_order.push(node);
}

fn dfs_second_pass(
    node: ConstraintId,
    reverse_adj: &HashMap<ConstraintId, Vec<ConstraintId>>,
    visited: &mut HashSet<ConstraintId>,
    component: &mut Vec<ConstraintId>,
) {
    visited.insert(node);
    component.push(node);
    
    if let Some(neighbors) = reverse_adj.get(&node) {
        for &neighbor in neighbors {
            if !visited.contains(&neighbor) {
                dfs_second_pass(neighbor, reverse_adj, visited, component);
            }
        }
    }
}

/// Check if the graph contains any cycles
pub fn has_cycle(graph: &ConstraintGraph) -> bool {
    !find_sccs(graph).is_dag
}

/// Find cycles in the graph (if any)
/// Returns list of cycles, where each cycle is a list of constraints
pub fn find_cycles(graph: &ConstraintGraph) -> Vec<Vec<ConstraintId>> {
    let analysis = find_sccs(graph);
    
    analysis
        .components
        .into_iter()
        .filter(|c| c.members.len() > 1)
        .map(|c| c.members)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn create_dag() -> ConstraintGraph {
        let mut graph = ConstraintGraph::new();
        
        for i in 0..3 {
            graph.add_wire(Wire::internal(WireId(i)));
        }
        
        let c_a = Constraint::new(ConstraintId(0), ConstraintType::Add, vec![], vec![WireId(0)]);
        let c_b = Constraint::new(ConstraintId(1), ConstraintType::Mul, vec![WireId(0)], vec![WireId(1)]);
        let c_c = Constraint::new(ConstraintId(2), ConstraintType::Add, vec![WireId(1)], vec![]);
        
        graph.add_constraint(c_a);
        graph.add_constraint(c_b);
        graph.add_constraint(c_c);
        
        graph.wires.get_mut(&WireId(0)).unwrap().producer = Some(ConstraintId(0));
        graph.wires.get_mut(&WireId(0)).unwrap().consumers.push(ConstraintId(1));
        
        graph.wires.get_mut(&WireId(1)).unwrap().producer = Some(ConstraintId(1));
        graph.wires.get_mut(&WireId(1)).unwrap().consumers.push(ConstraintId(2));
        
        graph.build_edges_from_wires();
        
        graph
    }

    #[test]
    fn test_dag_is_identified() {
        let graph = create_dag();
        let analysis = find_sccs(&graph);
        
        assert!(analysis.is_dag);
        assert_eq!(analysis.components.len(), 3);
    }

    #[test]
    fn test_no_cycles_in_dag() {
        let graph = create_dag();
        assert!(!has_cycle(&graph));
        assert!(find_cycles(&graph).is_empty());
    }
}