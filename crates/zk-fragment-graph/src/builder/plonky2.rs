//! Extract constraint graph from Plonky2 circuits
//! 
//! This module provides functionality to analyze Plonky2 circuits
//! and extract their dependency structure as a ConstraintGraph.

use crate::types::*;
use std::collections::HashMap;

use plonky2::field::extension::Extendable;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::field::types::Field;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::target::Target;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitData;
use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};

type F = GoldilocksField;
type C = PoseidonGoldilocksConfig;
const D: usize = 2;

/// Information extracted from a Plonky2 circuit
#[derive(Debug, Clone)]
pub struct Plonky2CircuitInfo {
    /// Number of gates/rows
    pub num_gates: usize,
    /// Number of wires/columns per row
    pub num_wires: usize,
    /// Number of public inputs
    pub num_public_inputs: usize,
    /// Gate types present
    pub gate_types: Vec<String>,
}

/// Extract basic circuit information from Plonky2 CircuitData
pub fn extract_circuit_info<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize>(
    circuit_data: &CircuitData<F, C, D>,
) -> Plonky2CircuitInfo {
    let common = &circuit_data.common;
    
    let gate_types: Vec<String> = common
        .gates
        .iter()
        .map(|g| format!("{:?}", g))
        .collect();
    
    Plonky2CircuitInfo {
        num_gates: common.gates.len(),
        num_wires: common.config.num_wires,
        num_public_inputs: common.num_public_inputs,
        gate_types,
    }
}

/// Builder for extracting constraint graphs from Plonky2 circuit builders
/// 
/// This tracks target dependencies during circuit construction
pub struct Plonky2GraphExtractor {
    /// Map from target to the "constraint" that produces it
    target_producer: HashMap<Target, ConstraintId>,
    /// Current constraint ID counter
    next_constraint_id: usize,
    /// The constraint graph being built
    graph: ConstraintGraph,
    /// Map from target to wire ID
    target_to_wire: HashMap<Target, WireId>,
    /// Next wire ID
    next_wire_id: usize,
}

impl Default for Plonky2GraphExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl Plonky2GraphExtractor {
    pub fn new() -> Self {
        Self {
            target_producer: HashMap::new(),
            next_constraint_id: 0,
            graph: ConstraintGraph::new(),
            target_to_wire: HashMap::new(),
            next_wire_id: 0,
        }
    }
    
    /// Get or create a wire ID for a target
    fn get_or_create_wire(&mut self, target: Target) -> WireId {
        if let Some(&wire_id) = self.target_to_wire.get(&target) {
            return wire_id;
        }
        
        let wire_id = WireId(self.next_wire_id);
        self.next_wire_id += 1;
        
        let wire = Wire::internal(wire_id);
        self.graph.add_wire(wire);
        self.target_to_wire.insert(target, wire_id);
        
        wire_id
    }
    
    /// Register a public input target
    pub fn register_public_input(&mut self, target: Target) {
        let wire_id = self.get_or_create_wire(target);
        if let Some(wire) = self.graph.wires.get_mut(&wire_id) {
            wire.role = WireRole::PublicInput;
        }
    }
    
    /// Register a public output target
    pub fn register_public_output(&mut self, target: Target) {
        let wire_id = self.get_or_create_wire(target);
        if let Some(wire) = self.graph.wires.get_mut(&wire_id) {
            wire.role = WireRole::PublicOutput;
        }
    }
    
    /// Register a constant target
    pub fn register_constant(&mut self, target: Target, value: &str) {
        let wire_id = self.get_or_create_wire(target);
        if let Some(wire) = self.graph.wires.get_mut(&wire_id) {
            wire.role = WireRole::Constant;
            wire.constant_value = Some(value.to_string());
        }
    }
    
    /// Register an operation that produces an output from inputs
    pub fn register_operation(
        &mut self,
        constraint_type: ConstraintType,
        inputs: &[Target],
        outputs: &[Target],
    ) -> ConstraintId {
        let constraint_id = ConstraintId(self.next_constraint_id);
        self.next_constraint_id += 1;
        
        let input_wires: Vec<WireId> = inputs
            .iter()
            .map(|&t| self.get_or_create_wire(t))
            .collect();
        
        let output_wires: Vec<WireId> = outputs
            .iter()
            .map(|&t| self.get_or_create_wire(t))
            .collect();
        
        // Update wire relationships
        for &wire_id in &output_wires {
            if let Some(wire) = self.graph.wires.get_mut(&wire_id) {
                wire.producer = Some(constraint_id);
            }
        }
        
        for &wire_id in &input_wires {
            if let Some(wire) = self.graph.wires.get_mut(&wire_id) {
                if !wire.consumers.contains(&constraint_id) {
                    wire.consumers.push(constraint_id);
                }
            }
        }
        
        // Track which constraint produces each output target
        for &target in outputs {
            self.target_producer.insert(target, constraint_id);
        }
        
        let constraint = Constraint::new(
            constraint_id,
            constraint_type,
            input_wires,
            output_wires,
        );
        
        self.graph.add_constraint(constraint);
        
        constraint_id
    }
    
    /// Build the final constraint graph
    pub fn build(mut self) -> ConstraintGraph {
        self.graph.build_edges_from_wires();
        self.graph
    }
}

/// Wrapper that instruments a Plonky2 CircuitBuilder to track dependencies
pub struct InstrumentedCircuitBuilder {
    builder: CircuitBuilder<F, D>,
    extractor: Plonky2GraphExtractor,
}

impl InstrumentedCircuitBuilder {
    pub fn new() -> Self {
        let config = plonky2::plonk::circuit_data::CircuitConfig::standard_recursion_config();
        Self {
            builder: CircuitBuilder::new(config),
            extractor: Plonky2GraphExtractor::new(),
        }
    }
    
    /// Add a virtual target (creates a new wire)
    pub fn add_virtual_target(&mut self) -> Target {
        let target = self.builder.add_virtual_target();
        self.extractor.get_or_create_wire(target);
        target
    }
    
    /// Add a public input
    pub fn add_virtual_public_input(&mut self) -> Target {
        let target = self.builder.add_virtual_public_input();
        self.extractor.register_public_input(target);
        target
    }
    
    /// Register a public input for an existing target
    pub fn register_public_input(&mut self, target: Target) {
        self.builder.register_public_input(target);
        self.extractor.register_public_input(target);
    }
    
    /// Add a constant
    pub fn constant(&mut self, value: F) -> Target {
        let target = self.builder.constant(value);
        self.extractor.register_constant(target, &format!("{:?}", value));
        target
    }
    
    /// Add two targets
    pub fn add(&mut self, a: Target, b: Target) -> Target {
        let output = self.builder.add(a, b);
        self.extractor.register_operation(
            ConstraintType::Add,
            &[a, b],
            &[output],
        );
        output
    }
    
    /// Multiply two targets
    pub fn mul(&mut self, a: Target, b: Target) -> Target {
        let output = self.builder.mul(a, b);
        self.extractor.register_operation(
            ConstraintType::Mul,
            &[a, b],
            &[output],
        );
        output
    }
    
    /// Subtract two targets
    pub fn sub(&mut self, a: Target, b: Target) -> Target {
        let output = self.builder.sub(a, b);
        self.extractor.register_operation(
            ConstraintType::Add, // Subtraction is addition with negation
            &[a, b],
            &[output],
        );
        output
    }
    
    /// Negate a target
    pub fn neg(&mut self, a: Target) -> Target {
        let output = self.builder.neg(a);
        self.extractor.register_operation(
            ConstraintType::Arithmetic,
            &[a],
            &[output],
        );
        output
    }
    
    /// Add many targets
    pub fn add_many(&mut self, terms: &[Target]) -> Target {
        if terms.is_empty() {
            return self.constant(F::ZERO);
        }
        if terms.len() == 1 {
            return terms[0];
        }
        
        let mut sum = terms[0];
        for &term in &terms[1..] {
            sum = self.add(sum, term);
        }
        sum
    }
    
    /// Multiply many targets
    pub fn mul_many(&mut self, terms: &[Target]) -> Target {
        if terms.is_empty() {
            return self.constant(F::ONE);
        }
        if terms.len() == 1 {
            return terms[0];
        }
        
        let mut product = terms[0];
        for &term in &terms[1..] {
            product = self.mul(product, term);
        }
        product
    }
    
    /// Get the underlying Plonky2 builder (for advanced operations)
    pub fn inner(&self) -> &CircuitBuilder<F, D> {
        &self.builder
    }
    
    /// Get mutable reference to underlying builder
    pub fn inner_mut(&mut self) -> &mut CircuitBuilder<F, D> {
        &mut self.builder
    }
    
    /// Build both the Plonky2 circuit and the constraint graph
    pub fn build(self) -> (CircuitData<F, C, D>, ConstraintGraph) {
        let circuit_data = self.builder.build::<C>();
        let graph = self.extractor.build();
        (circuit_data, graph)
    }
    
    /// Build only the constraint graph (doesn't build the actual circuit)
    pub fn build_graph(self) -> ConstraintGraph {
        self.extractor.build()
    }
}

impl Default for InstrumentedCircuitBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instrumented_builder_simple() {
        let mut builder = InstrumentedCircuitBuilder::new();
        
        let x = builder.add_virtual_public_input();
        let y = builder.add_virtual_public_input();
        let sum = builder.add(x, y);
        let product = builder.mul(sum, x);
        builder.register_public_input(product);
        
        let graph = builder.build_graph();
        
        assert_eq!(graph.constraints.len(), 2); // add and mul
        assert!(graph.is_dag());
    }

    #[test]
    fn test_instrumented_builder_chain() {
        let mut builder = InstrumentedCircuitBuilder::new();
        
        let mut current = builder.add_virtual_public_input();
        for _ in 0..5 {
            let one = builder.constant(F::ONE);
            current = builder.add(current, one);
        }
        builder.register_public_input(current);
        
        let graph = builder.build_graph();
        
        assert_eq!(graph.constraints.len(), 5);
        assert!(graph.is_dag());
    }

    #[test]
    fn test_full_circuit_build() {
        let mut builder = InstrumentedCircuitBuilder::new();
        
        let x = builder.add_virtual_public_input();
        let y = builder.add_virtual_public_input();
        let sum = builder.add(x, y);
        builder.register_public_input(sum);
        
        let (circuit_data, graph) = builder.build();
        
        // Check circuit was built
        assert_eq!(circuit_data.common.num_public_inputs, 3);
        
        // Check graph was extracted
        assert!(graph.is_dag());
    }
}