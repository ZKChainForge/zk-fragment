//! Fragment Proof Capsule data structures

use crate::commitment::{BoundaryCommitment, CommitmentOpening};
use crate::execution_hash::ExecutionHash;
use serde::{Serialize, Deserialize};

/// Metadata for a fragment being proven
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentMetadata {
    /// Unique fragment identifier
    pub fragment_id: u32,
    
    /// Number of constraints in this fragment
    pub constraint_count: u32,
    
    /// Number of input boundary wires
    pub input_boundary_count: u32,
    
    /// Number of output boundary wires
    pub output_boundary_count: u32,
    
    /// Fragment position in execution order
    pub execution_position: u32,
}

/// Input witness for a fragment
///
/// Contains all private information needed to prove this fragment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentWitness {
    /// Input boundary values and openings
    pub input_boundaries: Vec<CommitmentOpening>,
    
    /// Local witness values for this fragment's constraints
    /// These are indices into the original circuit witness
    pub local_witness: Vec<u64>,
    
    /// Public input values needed by this fragment
    pub public_inputs: Vec<u64>,
}

impl FragmentWitness {
    pub fn new() -> Self {
        FragmentWitness {
            input_boundaries: Vec::new(),
            local_witness: Vec::new(),
            public_inputs: Vec::new(),
        }
    }
    
    pub fn with_input_boundaries(mut self, boundaries: Vec<CommitmentOpening>) -> Self {
        self.input_boundaries = boundaries;
        self
    }
    
    pub fn with_local_witness(mut self, witness: Vec<u64>) -> Self {
        self.local_witness = witness;
        self
    }
    
    pub fn with_public_inputs(mut self, inputs: Vec<u64>) -> Self {
        self.public_inputs = inputs;
        self
    }
}

impl Default for FragmentWitness {
    fn default() -> Self {
        Self::new()
    }
}

/// Public outputs of a fragment proof
///
/// This is what gets revealed by the proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentProofOutput {
    /// Previous fragment's execution hash
    pub previous_execution_hash: ExecutionHash,
    
    /// This fragment's execution hash
    pub execution_hash: ExecutionHash,
    
    /// Output boundary commitments (for successors)
    pub output_boundaries: Vec<BoundaryCommitment>,
    
    /// Public outputs of this fragment (if any)
    pub public_outputs: Vec<u64>,
    
    /// Metadata about what was proven
    pub metadata: FragmentMetadata,
}

/// A complete proof for one fragment
///
/// This is what the prover produces and the aggregator consumes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentProof {
    /// Fragment metadata
    pub metadata: FragmentMetadata,
    
    /// The actual ZK proof bytes
    pub proof_bytes: Vec<u8>,
    
    /// Public outputs (includes execution hash, boundaries, etc.)
    pub public_outputs: FragmentProofOutput,
    
    /// Time taken to generate this proof (milliseconds)
    pub proving_time_ms: u64,
    
    /// Constraint count for this proof
    pub proof_constraint_count: u32,
}

impl FragmentProof {
    pub fn new(
        metadata: FragmentMetadata,
        proof_bytes: Vec<u8>,
        public_outputs: FragmentProofOutput,
    ) -> Self {
        FragmentProof {
            metadata,
            proof_bytes,
            public_outputs,
            proving_time_ms: 0,
            proof_constraint_count: 0,
        }
    }
    
    pub fn with_timing(mut self, time_ms: u64) -> Self {
        self.proving_time_ms = time_ms;
        self
    }
    
    pub fn with_constraint_count(mut self, count: u32) -> Self {
        self.proof_constraint_count = count;
        self
    }
}

/// Fragment Proof Capsule - the main structure
///
/// A complete, self-contained package for proving one fragment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentProofCapsule {
    /// Fragment metadata
    pub metadata: FragmentMetadata,
    
    /// Witness for proving this fragment
    pub witness: FragmentWitness,
    
    /// Generated proof (after proving)
    pub proof: Option<FragmentProof>,
    
    /// Whether proving was successful
    pub is_proven: bool,
    
    /// Error message if proving failed
    pub error: Option<String>,
}

impl FragmentProofCapsule {
    pub fn new(metadata: FragmentMetadata) -> Self {
        FragmentProofCapsule {
            metadata,
            witness: FragmentWitness::new(),
            proof: None,
            is_proven: false,
            error: None,
        }
    }
    
    pub fn with_witness(mut self, witness: FragmentWitness) -> Self {
        self.witness = witness;
        self
    }
    
    pub fn set_proof(&mut self, proof: FragmentProof) {
        self.proof = Some(proof);
        self.is_proven = true;
        self.error = None;
    }
    
    pub fn set_error(&mut self, error: String) {
        self.is_proven = false;
        self.error = Some(error);
    }
    
    /// Get proof reference
    pub fn get_proof(&self) -> Option<&FragmentProof> {
        self.proof.as_ref()
    }
    
    /// Get output boundaries from proof
    pub fn get_output_boundaries(&self) -> Option<Vec<BoundaryCommitment>> {
        self.proof
            .as_ref()
            .map(|p| p.public_outputs.output_boundaries.clone())
    }
    
    /// Get execution hash
    pub fn get_execution_hash(&self) -> Option<ExecutionHash> {
        self.proof
            .as_ref()
            .map(|p| p.public_outputs.execution_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fragment_metadata_creation() {
        let metadata = FragmentMetadata {
            fragment_id: 1,
            constraint_count: 100,
            input_boundary_count: 2,
            output_boundary_count: 2,
            execution_position: 0,
        };
        
        assert_eq!(metadata.fragment_id, 1);
        assert_eq!(metadata.constraint_count, 100);
    }

    #[test]
    fn test_fragment_witness_builder() {
        let witness = FragmentWitness::new()
            .with_local_witness(vec![1, 2, 3])
            .with_public_inputs(vec![4, 5]);
        
        assert_eq!(witness.local_witness.len(), 3);
        assert_eq!(witness.public_inputs.len(), 2);
    }

    #[test]
    fn test_fragment_proof_capsule() {
        let metadata = FragmentMetadata {
            fragment_id: 1,
            constraint_count: 100,
            input_boundary_count: 0,
            output_boundary_count: 1,
            execution_position: 0,
        };
        
        let capsule = FragmentProofCapsule::new(metadata);
        assert!(!capsule.is_proven);
        assert!(capsule.proof.is_none());
    }

    #[test]
    fn test_fragment_proof_capsule_set_error() {
        let metadata = FragmentMetadata {
            fragment_id: 1,
            constraint_count: 100,
            input_boundary_count: 0,
            output_boundary_count: 1,
            execution_position: 0,
        };
        
        let mut capsule = FragmentProofCapsule::new(metadata);
        capsule.set_error("Test error".to_string());
        
        assert!(!capsule.is_proven);
        assert_eq!(capsule.error, Some("Test error".to_string()));
    }
}