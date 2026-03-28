use crate::types::*;

/// Builder for manually constructing constraint graphs
/// Useful for testing and simple circuits
pub struct GraphBuilder {
    graph: ConstraintGraph,
    next_constraint_id: usize,
    next_wire_id: usize,
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            graph: ConstraintGraph::new(),
            next_constraint_id: 0,
            next_wire_id: 0,
        }
    }
    
    /// Add a public input wire
    pub fn add_public_input(&mut self) -> WireId {
        let id = WireId(self.next_wire_id);
        self.next_wire_id += 1;
        
        let wire = Wire::public_input(id);
        self.graph.add_wire(wire);
        
        id
    }
    
    /// Add a private witness wire
    pub fn add_private_witness(&mut self) -> WireId {
        let id = WireId(self.next_wire_id);
        self.next_wire_id += 1;
        
        let wire = Wire::private_witness(id);
        self.graph.add_wire(wire);
        
        id
    }
    
    /// Add a constant wire
    pub fn add_constant(&mut self, value: &str) -> WireId {
        let id = WireId(self.next_wire_id);
        self.next_wire_id += 1;
        
        let wire = Wire::constant(id, value.to_string());
        self.graph.add_wire(wire);
        
        id
    }
    
    /// Add an internal wire (allocated but not yet assigned)
    fn add_internal_wire(&mut self) -> WireId {
        let id = WireId(self.next_wire_id);
        self.next_wire_id += 1;
        
        let wire = Wire::internal(id);
        self.graph.add_wire(wire);
        
        id
    }
    
    /// Add an addition constraint: output = input1 + input2
    pub fn add_addition(&mut self, input1: WireId, input2: WireId) -> WireId {
        let output = self.add_internal_wire();
        let id = ConstraintId(self.next_constraint_id);
        self.next_constraint_id += 1;
        
        let constraint = Constraint::new(
            id,
            ConstraintType::Add,
            vec![input1, input2],
            vec![output],
        );
        
        // Update wire relationships
        self.graph.wires.get_mut(&output).unwrap().producer = Some(id);
        if let Some(w) = self.graph.wires.get_mut(&input1) {
            w.consumers.push(id);
        }
        if let Some(w) = self.graph.wires.get_mut(&input2) {
            w.consumers.push(id);
        }
        
        self.graph.add_constraint(constraint);
        
        output
    }
    
    /// Add a multiplication constraint: output = input1 * input2
    pub fn add_multiplication(&mut self, input1: WireId, input2: WireId) -> WireId {
        let output = self.add_internal_wire();
        let id = ConstraintId(self.next_constraint_id);
        self.next_constraint_id += 1;
        
        let constraint = Constraint::new(
            id,
            ConstraintType::Mul,
            vec![input1, input2],
            vec![output],
        );
        
        self.graph.wires.get_mut(&output).unwrap().producer = Some(id);
        if let Some(w) = self.graph.wires.get_mut(&input1) {
            w.consumers.push(id);
        }
        if let Some(w) = self.graph.wires.get_mut(&input2) {
            w.consumers.push(id);
        }
        
        self.graph.add_constraint(constraint);
        
        output
    }
    
    /// Mark a wire as public output
    pub fn mark_public_output(&mut self, wire: WireId) {
        if let Some(w) = self.graph.wires.get_mut(&wire) {
            w.role = WireRole::PublicOutput;
        }
    }
    
    /// Add a generic constraint
    pub fn add_constraint(
        &mut self,
        constraint_type: ConstraintType,
        inputs: Vec<WireId>,
        num_outputs: usize,
    ) -> Vec<WireId> {
        let outputs: Vec<WireId> = (0..num_outputs)
            .map(|_| self.add_internal_wire())
            .collect();
        
        let id = ConstraintId(self.next_constraint_id);
        self.next_constraint_id += 1;
        
        let constraint = Constraint::new(id, constraint_type, inputs.clone(), outputs.clone());
        
        for &output in &outputs {
            self.graph.wires.get_mut(&output).unwrap().producer = Some(id);
        }
        for &input in &inputs {
            if let Some(w) = self.graph.wires.get_mut(&input) {
                w.consumers.push(id);
            }
        }
        
        self.graph.add_constraint(constraint);
        
        outputs
    }
    
    /// Build the final graph
    pub fn build(mut self) -> ConstraintGraph {
        self.graph.build_edges_from_wires();
        self.graph
    }
}

/// Create a simple chain circuit: x -> f -> f -> f -> ... -> output
pub fn create_chain_circuit(length: usize) -> ConstraintGraph {
    let mut builder = GraphBuilder::new();
    
    let input = builder.add_public_input();
    let mut current = input;
    
    for _ in 0..length {
        let one = builder.add_constant("1");
        current = builder.add_addition(current, one);
    }
    
    builder.mark_public_output(current);
    builder.build()
}

/// Create a binary tree circuit
pub fn create_tree_circuit(depth: usize) -> ConstraintGraph {
    let mut builder = GraphBuilder::new();
    
    // Create 2^depth leaves as inputs
    let num_leaves = 1 << depth;
    let mut current_level: Vec<WireId> = (0..num_leaves)
        .map(|_| builder.add_public_input())
        .collect();
    
    // Build tree by combining pairs
    while current_level.len() > 1 {
        let mut next_level = Vec::new();
        for chunk in current_level.chunks(2) {
            if chunk.len() == 2 {
                let combined = builder.add_addition(chunk[0], chunk[1]);
                next_level.push(combined);
            } else {
                next_level.push(chunk[0]);
            }
        }
        current_level = next_level;
    }
    
    if let Some(&output) = current_level.first() {
        builder.mark_public_output(output);
    }
    
    builder.build()
}

/// Create a diamond pattern circuit
/// Multiple paths from input to output
pub fn create_diamond_circuit(width: usize, depth: usize) -> ConstraintGraph {
    let mut builder = GraphBuilder::new();
    
    let input = builder.add_public_input();
    
    // Create initial fan-out
    let mut current_level: Vec<WireId> = (0..width)
        .map(|_| {
            let one = builder.add_constant("1");
            builder.add_addition(input, one)
        })
        .collect();
    
    // Middle layers
    for _ in 0..depth {
        let mut next_level = Vec::new();
        for i in 0..width {
            let left = current_level[i];
            let right = current_level[(i + 1) % width];
            let combined = builder.add_multiplication(left, right);
            next_level.push(combined);
        }
        current_level = next_level;
    }
    
    // Final fan-in
    let mut output = current_level[0];
    for i in 1..current_level.len() {
        output = builder.add_addition(output, current_level[i]);
    }
    
    builder.mark_public_output(output);
    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithms;

    #[test]
    fn test_graph_builder_simple() {
        let mut builder = GraphBuilder::new();
        
        let x = builder.add_public_input();
        let y = builder.add_public_input();
        let sum = builder.add_addition(x, y);
        builder.mark_public_output(sum);
        
        let graph = builder.build();
        
        assert_eq!(graph.constraints.len(), 1);
        assert_eq!(graph.wires.len(), 3);
        assert!(graph.is_dag());
    }

    #[test]
    fn test_chain_circuit() {
        let graph = create_chain_circuit(5);
        
        assert_eq!(graph.constraints.len(), 5);
        assert!(graph.is_dag());
        
        let order = algorithms::topological_sort(&graph).unwrap();
        assert_eq!(order.len(), 5);
    }

    #[test]
    fn test_tree_circuit() {
        let graph = create_tree_circuit(3);
        
        assert!(graph.is_dag());
        
        // 2^3 = 8 leaves, 7 internal nodes
        assert_eq!(graph.constraints.len(), 7);
    }

    #[test]
    fn test_diamond_circuit() {
        let graph = create_diamond_circuit(4, 2);
        
        assert!(graph.is_dag());
        assert!(graph.constraints.len() > 0);
    }
}