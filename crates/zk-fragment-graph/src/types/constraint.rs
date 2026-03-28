use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for a constraint in the circuit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ConstraintId(pub usize);

impl fmt::Display for ConstraintId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "C{}", self.0)
    }
}

impl From<usize> for ConstraintId {
    fn from(id: usize) -> Self {
        ConstraintId(id)
    }
}

/// Type of gate/constraint in the circuit
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintType {
    /// Addition: a + b = c
    Add,
    /// Multiplication: a * b = c
    Mul,
    /// Constant: wire = constant_value
    Constant,
    /// Public input constraint
    PublicInput,
    /// Public output constraint
    PublicOutput,
    /// Poseidon hash round
    Poseidon,
    /// Range check
    RangeCheck,
    /// Arithmetic gate (generic)
    Arithmetic,
    /// Custom gate with name
    Custom(String),
    /// Unknown/other
    Unknown,
}

impl Default for ConstraintType {
    fn default() -> Self {
        ConstraintType::Unknown
    }
}

/// Metadata about a constraint's resource requirements
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConstraintMetadata {
    /// Estimated memory in bytes for proving this constraint
    pub estimated_memory: u64,
    /// Estimated time in microseconds for proving
    pub estimated_time_us: u64,
    /// Number of field operations
    pub field_operations: usize,
    /// Row index in the circuit (if applicable)
    pub row_index: Option<usize>,
}

/// A single constraint in the circuit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    /// Unique identifier
    pub id: ConstraintId,
    /// Type of constraint
    pub constraint_type: ConstraintType,
    /// Input wire IDs (wires this constraint reads)
    pub input_wires: Vec<super::WireId>,
    /// Output wire IDs (wires this constraint produces)
    pub output_wires: Vec<super::WireId>,
    /// Resource metadata
    pub metadata: ConstraintMetadata,
}

impl Constraint {
    /// Create a new constraint
    pub fn new(
        id: ConstraintId,
        constraint_type: ConstraintType,
        input_wires: Vec<super::WireId>,
        output_wires: Vec<super::WireId>,
    ) -> Self {
        Self {
            id,
            constraint_type,
            input_wires,
            output_wires,
            metadata: ConstraintMetadata::default(),
        }
    }

    /// Total number of wires involved
    pub fn wire_count(&self) -> usize {
        self.input_wires.len() + self.output_wires.len()
    }

    /// Check if this constraint uses a specific wire as input
    pub fn uses_wire(&self, wire_id: super::WireId) -> bool {
        self.input_wires.contains(&wire_id)
    }

    /// Check if this constraint produces a specific wire
    pub fn produces_wire(&self, wire_id: super::WireId) -> bool {
        self.output_wires.contains(&wire_id)
    }
}