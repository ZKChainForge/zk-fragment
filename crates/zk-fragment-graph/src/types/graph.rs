use super::{Constraint, ConstraintId, Wire, WireId, WireRole};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use std::collections::{HashMap, HashSet};

/// Edge in the constraint graph representing data flow
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Edge {
    /// Source constraint (producer)
    pub from: ConstraintId,
    /// Target constraint (consumer)
    pub to: ConstraintId,
    /// Wire that carries data between constraints
    pub wire: WireId,
}

impl Edge {
    pub fn new(from: ConstraintId, to: ConstraintId, wire: WireId) -> Self {
        Self { from, to, wire }
    }
}

/// Statistics about the constraint graph
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphStats {
    /// Total number of constraints
    pub constraint_count: usize,
    /// Total number of wires
    pub wire_count: usize,
    /// Total number of edges (dependencies)
    pub edge_count: usize,
    /// Number of public input wires
    pub public_input_count: usize,
    /// Number of public output wires
    pub public_output_count: usize,
    /// Number of private witness wires
    pub private_witness_count: usize,
    /// Maximum in-degree (most inputs to a single constraint)
    pub max_in_degree: usize,
    /// Maximum out-degree (most constraints depending on single constraint)
    pub max_out_degree: usize,
    /// Graph density (edges / possible edges)
    pub density: f64,
    /// Longest path in the graph (circuit depth)
    pub longest_path: usize,
}

/// The main constraint graph structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintGraph {
    /// All constraints indexed by ID
    pub constraints: IndexMap<ConstraintId, Constraint>,
    /// All wires indexed by ID
    pub wires: IndexMap<WireId, Wire>,
    /// Edges representing dependencies
    pub edges: Vec<Edge>,
    
    // Precomputed structures for efficient queries
    /// Adjacency list: constraint -> constraints it depends on (predecessors)
    #[serde(skip)]
    predecessors: HashMap<ConstraintId, Vec<ConstraintId>>,
    /// Adjacency list: constraint -> constraints that depend on it (successors)
    #[serde(skip)]
    successors: HashMap<ConstraintId, Vec<ConstraintId>>,
    /// Topological order (computed lazily)
    #[serde(skip)]
    topological_order: Option<Vec<ConstraintId>>,
    /// Cut vertices (computed lazily)
    #[serde(skip)]
    cut_vertices: Option<Vec<ConstraintId>>,
}

impl Default for ConstraintGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl ConstraintGraph {
    /// Create a new empty constraint graph
    pub fn new() -> Self {
        Self {
            constraints: IndexMap::new(),
            wires: IndexMap::new(),
            edges: Vec::new(),
            predecessors: HashMap::new(),
            successors: HashMap::new(),
            topological_order: None,
            cut_vertices: None,
        }
    }

    /// Add a constraint to the graph
    pub fn add_constraint(&mut self, constraint: Constraint) {
        let id = constraint.id;
        
        // Ensure adjacency lists exist
        self.predecessors.entry(id).or_default();
        self.successors.entry(id).or_default();
        
        // Update wire information
        for &wire_id in &constraint.output_wires {
            if let Some(wire) = self.wires.get_mut(&wire_id) {
                wire.producer = Some(id);
            }
        }
        
        for &wire_id in &constraint.input_wires {
            if let Some(wire) = self.wires.get_mut(&wire_id) {
                if !wire.consumers.contains(&id) {
                    wire.consumers.push(id);
                }
            }
        }
        
        self.constraints.insert(id, constraint);
        
        // Invalidate cached computations
        self.topological_order = None;
        self.cut_vertices = None;
    }

    /// Add a wire to the graph
    pub fn add_wire(&mut self, wire: Wire) {
        self.wires.insert(wire.id, wire);
    }

    /// Add an edge (dependency) between constraints
    pub fn add_edge(&mut self, from: ConstraintId, to: ConstraintId, wire: WireId) {
        let edge = Edge::new(from, to, wire);
        
        if !self.edges.contains(&edge) {
            self.edges.push(edge);
            
            // Update adjacency lists
            self.successors.entry(from).or_default().push(to);
            self.predecessors.entry(to).or_default().push(from);
            
            // Invalidate cached computations
            self.topological_order = None;
            self.cut_vertices = None;
        }
    }

    /// Build edges from wire producer/consumer relationships
    pub fn build_edges_from_wires(&mut self) {
        self.edges.clear();
        self.predecessors.clear();
        self.successors.clear();
        
        // Initialize adjacency lists for all constraints
        for &id in self.constraints.keys() {
            self.predecessors.entry(id).or_default();
            self.successors.entry(id).or_default();
        }
        
        // For each wire with a producer, create edges to all consumers
        for wire in self.wires.values() {
            if let Some(producer) = wire.producer {
                for &consumer in &wire.consumers {
                    if producer != consumer {
                        let edge = Edge::new(producer, consumer, wire.id);
                        if !self.edges.contains(&edge) {
                            self.edges.push(edge);
                            self.successors.entry(producer).or_default().push(consumer);
                            self.predecessors.entry(consumer).or_default().push(producer);
                        }
                    }
                }
            }
        }
        
        // Invalidate cached computations
        self.topological_order = None;
        self.cut_vertices = None;
    }

    /// Get predecessors (constraints this one depends on)
    pub fn get_predecessors(&self, id: ConstraintId) -> &[ConstraintId] {
        self.predecessors.get(&id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get successors (constraints that depend on this one)
    pub fn get_successors(&self, id: ConstraintId) -> &[ConstraintId] {
        self.successors.get(&id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get in-degree (number of dependencies)
    pub fn in_degree(&self, id: ConstraintId) -> usize {
        self.predecessors.get(&id).map(|v| v.len()).unwrap_or(0)
    }

    /// Get out-degree (number of dependents)
    pub fn out_degree(&self, id: ConstraintId) -> usize {
        self.successors.get(&id).map(|v| v.len()).unwrap_or(0)
    }

    /// Get all constraints with no predecessors (entry points)
    pub fn get_sources(&self) -> Vec<ConstraintId> {
        self.constraints
            .keys()
            .filter(|&&id| self.in_degree(id) == 0)
            .copied()
            .collect()
    }

    /// Get all constraints with no successors (exit points)
    pub fn get_sinks(&self) -> Vec<ConstraintId> {
        self.constraints
            .keys()
            .filter(|&&id| self.out_degree(id) == 0)
            .copied()
            .collect()
    }

    /// Get public input wires
    pub fn get_public_inputs(&self) -> Vec<WireId> {
        self.wires
            .values()
            .filter(|w| matches!(w.role, WireRole::PublicInput))
            .map(|w| w.id)
            .collect()
    }

    /// Get public output wires
    pub fn get_public_outputs(&self) -> Vec<WireId> {
        self.wires
            .values()
            .filter(|w| matches!(w.role, WireRole::PublicOutput))
            .map(|w| w.id)
            .collect()
    }

    /// Compute graph statistics
    pub fn compute_stats(&self) -> GraphStats {
        let constraint_count = self.constraints.len();
        let wire_count = self.wires.len();
        let edge_count = self.edges.len();
        
        let public_input_count = self.wires.values()
            .filter(|w| matches!(w.role, WireRole::PublicInput))
            .count();
        let public_output_count = self.wires.values()
            .filter(|w| matches!(w.role, WireRole::PublicOutput))
            .count();
        let private_witness_count = self.wires.values()
            .filter(|w| matches!(w.role, WireRole::PrivateWitness))
            .count();
        
        let max_in_degree = self.constraints.keys()
            .map(|&id| self.in_degree(id))
            .max()
            .unwrap_or(0);
        let max_out_degree = self.constraints.keys()
            .map(|&id| self.out_degree(id))
            .max()
            .unwrap_or(0);
        
        let possible_edges = if constraint_count > 1 {
            constraint_count * (constraint_count - 1)
        } else {
            1
        };
        let density = edge_count as f64 / possible_edges as f64;
        
        // Compute longest path using topological sort
        let longest_path = self.compute_longest_path();
        
        GraphStats {
            constraint_count,
            wire_count,
            edge_count,
            public_input_count,
            public_output_count,
            private_witness_count,
            max_in_degree,
            max_out_degree,
            density,
            longest_path,
        }
    }

    /// Compute the longest path in the DAG
    fn compute_longest_path(&self) -> usize {
        if self.constraints.is_empty() {
            return 0;
        }
        
        let mut dist: HashMap<ConstraintId, usize> = HashMap::new();
        
        // Initialize distances
        for &id in self.constraints.keys() {
            dist.insert(id, 0);
        }
        
        // Process in topological order
        if let Ok(order) = crate::algorithms::topological_sort(self) {
            for id in order {
                let current_dist = dist[&id];
                for &successor in self.get_successors(id) {
                    let new_dist = current_dist + 1;
                    if new_dist > dist[&successor] {
                        dist.insert(successor, new_dist);
                    }
                }
            }
        }
        
        dist.values().copied().max().unwrap_or(0)
    }

    /// Check if the graph is a valid DAG (no cycles)
    pub fn is_dag(&self) -> bool {
        crate::algorithms::topological_sort(self).is_ok()
    }

    /// Get the number of constraints
    pub fn constraint_count(&self) -> usize {
        self.constraints.len()
    }

    /// Get the number of wires
    pub fn wire_count(&self) -> usize {
        self.wires.len()
    }

    /// Get the number of edges
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}