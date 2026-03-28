use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for a wire in the circuit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WireId(pub usize);

impl fmt::Display for WireId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "W{}", self.0)
    }
}

impl From<usize> for WireId {
    fn from(id: usize) -> Self {
        WireId(id)
    }
}

/// Classification of a wire's role in the circuit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WireRole {
    /// Public input provided by verifier
    PublicInput,
    /// Public output revealed to verifier
    PublicOutput,
    /// Private witness known only to prover
    PrivateWitness,
    /// Internally computed intermediate value
    Internal,
    /// Constant value
    Constant,
}

impl Default for WireRole {
    fn default() -> Self {
        WireRole::Internal
    }
}

/// Complete information about a wire
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wire {
    /// Unique identifier
    pub id: WireId,
    /// Role/classification of this wire
    pub role: WireRole,
    /// Constraint that produces this wire's value (None for inputs/constants)
    pub producer: Option<super::ConstraintId>,
    /// Constraints that consume this wire's value
    pub consumers: Vec<super::ConstraintId>,
    /// If this is a constant, store the value (as string for serialization)
    pub constant_value: Option<String>,
}

impl Wire {
    /// Create a new wire
    pub fn new(id: WireId, role: WireRole) -> Self {
        Self {
            id,
            role,
            producer: None,
            consumers: Vec::new(),
            constant_value: None,
        }
    }

    /// Create a public input wire
    pub fn public_input(id: WireId) -> Self {
        Self::new(id, WireRole::PublicInput)
    }

    /// Create a public output wire
    pub fn public_output(id: WireId) -> Self {
        Self::new(id, WireRole::PublicOutput)
    }

    /// Create a private witness wire
    pub fn private_witness(id: WireId) -> Self {
        Self::new(id, WireRole::PrivateWitness)
    }

    /// Create an internal wire
    pub fn internal(id: WireId) -> Self {
        Self::new(id, WireRole::Internal)
    }

    /// Create a constant wire
    pub fn constant(id: WireId, value: String) -> Self {
        let mut wire = Self::new(id, WireRole::Constant);
        wire.constant_value = Some(value);
        wire
    }

    /// Check if this wire has a producer constraint
    pub fn has_producer(&self) -> bool {
        self.producer.is_some()
    }

    /// Check if this wire has any consumers
    pub fn has_consumers(&self) -> bool {
        !self.consumers.is_empty()
    }

    /// Number of consumers
    pub fn consumer_count(&self) -> usize {
        self.consumers.len()
    }

    /// Check if this wire is an input to the circuit (no producer)
    pub fn is_circuit_input(&self) -> bool {
        matches!(self.role, WireRole::PublicInput | WireRole::PrivateWitness | WireRole::Constant)
    }

    /// Check if this wire is an output of the circuit
    pub fn is_circuit_output(&self) -> bool {
        matches!(self.role, WireRole::PublicOutput)
    }
}