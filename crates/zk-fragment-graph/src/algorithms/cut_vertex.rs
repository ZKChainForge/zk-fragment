use crate::types::{ConstraintGraph, ConstraintId};
use std::collections::{HashMap, HashSet};

/// Result of cut vertex analysis
#[derive(Debug, Clone)]
pub struct CutVertexAnalysis {
    /// Constraints that are cut vertices
    pub cut_vertices: Vec<ConstraintId>,
    /// Discovery time for each constraint (DFS order)
    pub discovery_time: HashMap<ConstraintId, usize>,
    /// Lowest reachable discovery time for each constraint
    pub low: HashMap<ConstraintId, usize>,
    /// Parent in DFS tree
    pub parent: HashMap<ConstraintId, Option<ConstraintId>>,
}

/// Find cut vertices (articulation points) in the constraint graph
/// 
/// A cut vertex is a vertex whose removal would disconnect the graph.
/// These are natural fragmentation points with minimal boundary cost.
/// 
/// Note: This treats the directed graph as undirected for cut vertex analysis.
pub fn find_cut_vertices(graph: &ConstraintGraph) -> CutVertexAnalysis {
    let mut analysis = CutVertexAnalysis {
        cut_vertices: Vec::new(),
        discovery_time: HashMap::new(),
        low: HashMap::new(),
        parent: HashMap::new(),
    };
    
    if graph.constraints.is_empty() {
        return analysis;
    }
    
    // Build undirected adjacency list
    let mut adj: HashMap<ConstraintId, HashSet<ConstraintId>> = HashMap::new();
    for &id in graph.constraints.keys() {
        adj.insert(id, HashSet::new());
    }
    
    for &id in graph.constraints.keys() {
        for &succ in graph.get_successors(id) {
            adj.get_mut(&id).unwrap().insert(succ);
            adj.get_mut(&succ).unwrap().insert(id);
        }
    }
    
    let mut visited: HashSet<ConstraintId> = HashSet::new();
    let mut time: usize = 0;
    let mut cut_vertices: HashSet<ConstraintId> = HashSet::new();
    
    // Run DFS from each unvisited node (handles disconnected components)
    for &start in graph.constraints.keys() {
        if !visited.contains(&start) {
            dfs_cut_vertex(
                start,
                None,
                &adj,
                &mut visited,
                &mut time,
                &mut analysis.discovery_time,
                &mut analysis.low,
                &mut analysis.parent,
                &mut cut_vertices,
            );
        }
    }
    
    analysis.cut_vertices = cut_vertices.into_iter().collect();
    analysis.cut_vertices.sort_by_key(|c| c.0);
    
    analysis
}

fn dfs_cut_vertex(
    node: ConstraintId,
    parent: Option<ConstraintId>,
    adj: &HashMap<ConstraintId, HashSet<ConstraintId>>,
    visited: &mut HashSet<ConstraintId>,
    time: &mut usize,
    discovery: &mut HashMap<ConstraintId, usize>,
    low: &mut HashMap<ConstraintId, usize>,
    parent_map: &mut HashMap<ConstraintId, Option<ConstraintId>>,
    cut_vertices: &mut HashSet<ConstraintId>,
) {
    visited.insert(node);
    discovery.insert(node, *time);
    low.insert(node, *time);
    parent_map.insert(node, parent);
    *time += 1;
    
    let mut children = 0;
    
    if let Some(neighbors) = adj.get(&node) {
        for &neighbor in neighbors {
            if !visited.contains(&neighbor) {
                children += 1;
                dfs_cut_vertex(
                    neighbor,
                    Some(node),
                    adj,
                    visited,
                    time,
                    discovery,
                    low,
                    parent_map,
                    cut_vertices,
                );
                
                // Update low value
                let neighbor_low = low[&neighbor];
                let node_low = low[&node];
                low.insert(node, node_low.min(neighbor_low));
                
                // Check if node is a cut vertex
                // Case 1: node is root of DFS tree and has two or more children
                if parent.is_none() && children > 1 {
                    cut_vertices.insert(node);
                }
                
                // Case 2: node is not root and low value of child >= discovery time of node
                if parent.is_some() && low[&neighbor] >= discovery[&node] {
                    cut_vertices.insert(node);
                }
            } else if Some(neighbor) != parent {
                // Back edge: update low value
                let neighbor_disc = discovery[&neighbor];
                let node_low = low[&node];
                low.insert(node, node_low.min(neighbor_disc));
            }
        }
    }
}

/// Find bridges (cut edges) in the graph
/// A bridge is an edge whose removal would disconnect the graph
pub fn find_bridges(graph: &ConstraintGraph) -> Vec<(ConstraintId, ConstraintId)> {
    if graph.constraints.is_empty() {
        return Vec::new();
    }
    
    // Build undirected adjacency list
    let mut adj: HashMap<ConstraintId, HashSet<ConstraintId>> = HashMap::new();
    for &id in graph.constraints.keys() {
        adj.insert(id, HashSet::new());
    }
    
    for &id in graph.constraints.keys() {
        for &succ in graph.get_successors(id) {
            adj.get_mut(&id).unwrap().insert(succ);
            adj.get_mut(&succ).unwrap().insert(id);
        }
    }
    
    let mut visited: HashSet<ConstraintId> = HashSet::new();
    let mut time: usize = 0;
    let mut discovery: HashMap<ConstraintId, usize> = HashMap::new();
    let mut low: HashMap<ConstraintId, usize> = HashMap::new();
    let mut bridges: Vec<(ConstraintId, ConstraintId)> = Vec::new();
    
    for &start in graph.constraints.keys() {
        if !visited.contains(&start) {
            dfs_bridge(
                start,
                None,
                &adj,
                &mut visited,
                &mut time,
                &mut discovery,
                &mut low,
                &mut bridges,
            );
        }
    }
    
    bridges
}

fn dfs_bridge(
    node: ConstraintId,
    parent: Option<ConstraintId>,
    adj: &HashMap<ConstraintId, HashSet<ConstraintId>>,
    visited: &mut HashSet<ConstraintId>,
    time: &mut usize,
    discovery: &mut HashMap<ConstraintId, usize>,
    low: &mut HashMap<ConstraintId, usize>,
    bridges: &mut Vec<(ConstraintId, ConstraintId)>,
) {
    visited.insert(node);
    discovery.insert(node, *time);
    low.insert(node, *time);
    *time += 1;
    
    if let Some(neighbors) = adj.get(&node) {
        for &neighbor in neighbors {
            if !visited.contains(&neighbor) {
                dfs_bridge(
                    neighbor,
                    Some(node),
                    adj,
                    visited,
                    time,
                    discovery,
                    low,
                    bridges,
                );
                
                let neighbor_low = low[&neighbor];
                let node_low = low[&node];
                low.insert(node, node_low.min(neighbor_low));
                
                // If low value of neighbor > discovery time of node, it's a bridge
                if low[&neighbor] > discovery[&node] {
                    bridges.push((node, neighbor));
                }
            } else if Some(neighbor) != parent {
                let neighbor_disc = discovery[&neighbor];
                let node_low = low[&node];
                low.insert(node, node_low.min(neighbor_disc));
            }
        }
    }
}

/// Score how good a constraint is as a fragmentation point
/// Higher score = better cut point
pub fn score_cut_candidates(graph: &ConstraintGraph) -> HashMap<ConstraintId, f64> {
    let analysis = find_cut_vertices(graph);
    let bridges = find_bridges(graph);
    
    let mut scores: HashMap<ConstraintId, f64> = HashMap::new();
    
    for &id in graph.constraints.keys() {
        let mut score = 0.0;
        
        // Cut vertices get a base score
        if analysis.cut_vertices.contains(&id) {
            score += 10.0;
        }
        
        // Check if this node is incident to a bridge
        let is_bridge_endpoint = bridges.iter().any(|(a, b)| *a == id || *b == id);
        if is_bridge_endpoint {
            score += 5.0;
        }
        
        // Lower connectivity = better cut point
        let in_deg = graph.in_degree(id);
        let out_deg = graph.out_degree(id);
        let total_degree = in_deg + out_deg;
        
        if total_degree > 0 {
            // Prefer nodes with lower degree (fewer wires to handle at boundary)
            score += 1.0 / (total_degree as f64);
        }
        
        // Balanced position in graph is better
        if let Some(&disc) = analysis.discovery_time.get(&id) {
            let max_disc = analysis.discovery_time.values().copied().max().unwrap_or(1);
            let position = disc as f64 / max_disc as f64;
            // Prefer middle positions
            let balance_score = 1.0 - (2.0 * position - 1.0).abs();
            score += balance_score;
        }
        
        scores.insert(id, score);
    }
    
    scores
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn create_graph_with_cut_vertex() -> ConstraintGraph {
        // A -- B -- C
        //      |
        //      D -- E
        // B is a cut vertex
        let mut graph = ConstraintGraph::new();
        
        for i in 0..5 {
            graph.add_wire(Wire::internal(WireId(i)));
        }
        
        // A
        let c_a = Constraint::new(ConstraintId(0), ConstraintType::Add, vec![], vec![WireId(0)]);
        // B
        let c_b = Constraint::new(ConstraintId(1), ConstraintType::Mul, vec![WireId(0)], vec![WireId(1), WireId(2)]);
        // C
        let c_c = Constraint::new(ConstraintId(2), ConstraintType::Add, vec![WireId(1)], vec![]);
        // D
        let c_d = Constraint::new(ConstraintId(3), ConstraintType::Add, vec![WireId(2)], vec![WireId(3)]);
        // E
        let c_e = Constraint::new(ConstraintId(4), ConstraintType::Add, vec![WireId(3)], vec![]);
        
        graph.add_constraint(c_a);
        graph.add_constraint(c_b);
        graph.add_constraint(c_c);
        graph.add_constraint(c_d);
        graph.add_constraint(c_e);
        
        // Set up wire relationships
        graph.wires.get_mut(&WireId(0)).unwrap().producer = Some(ConstraintId(0));
        graph.wires.get_mut(&WireId(0)).unwrap().consumers.push(ConstraintId(1));
        
        graph.wires.get_mut(&WireId(1)).unwrap().producer = Some(ConstraintId(1));
        graph.wires.get_mut(&WireId(1)).unwrap().consumers.push(ConstraintId(2));
        
        graph.wires.get_mut(&WireId(2)).unwrap().producer = Some(ConstraintId(1));
        graph.wires.get_mut(&WireId(2)).unwrap().consumers.push(ConstraintId(3));
        
        graph.wires.get_mut(&WireId(3)).unwrap().producer = Some(ConstraintId(3));
        graph.wires.get_mut(&WireId(3)).unwrap().consumers.push(ConstraintId(4));
        
        graph.build_edges_from_wires();
        
        graph
    }

    #[test]
    fn test_find_cut_vertices() {
        let graph = create_graph_with_cut_vertex();
        let analysis = find_cut_vertices(&graph);
        
        // B (ConstraintId(1)) should be a cut vertex
        assert!(analysis.cut_vertices.contains(&ConstraintId(1)));
        
        // D (ConstraintId(3)) is also a cut vertex (separates E)
        assert!(analysis.cut_vertices.contains(&ConstraintId(3)));
    }

    #[test]
    fn test_find_bridges() {
        let graph = create_graph_with_cut_vertex();
        let bridges = find_bridges(&graph);
        
        // All edges in this linear-ish graph should be bridges
        assert!(!bridges.is_empty());
    }

    #[test]
    fn test_score_cut_candidates() {
        let graph = create_graph_with_cut_vertex();
        let scores = score_cut_candidates(&graph);
        
        // Cut vertices should have higher scores
        let cut_score = scores[&ConstraintId(1)];
        let non_cut_score = scores[&ConstraintId(0)];
        
        assert!(cut_score > non_cut_score);
    }
}