//! Week 3 test suite for fragment proving system
//!
//! This module contains comprehensive tests for:
//! - Commitment schemes (Poseidon hashing)
//! - Witness management and partitioning
//! - Circuit gadgets and boundaries
//! - Fragment proving and parallel utilities
//! - End-to-end integration workflows

#[cfg(test)]
pub mod commitment_tests;

#[cfg(test)]
pub mod integration_tests;

#[cfg(test)]
pub mod boundary_verification_tests;

#[cfg(test)]
pub mod witness_tests;

#[cfg(test)]
pub mod prover_tests;

// Re-export test types for convenience
#[cfg(test)]
pub use crate::commitment::*;
#[cfg(test)]
pub use crate::execution_hash::*;
#[cfg(test)]
pub use crate::capsule::*;
#[cfg(test)]
pub use crate::witness::*;
#[cfg(test)]
pub use crate::circuit::*;
#[cfg(test)]
pub use crate::prover::*;
#[cfg(test)]
pub use crate::proof::*;

/// Test utilities
#[cfg(test)]
pub mod utilities {
    use crate::*;

    /// Create a simple test proof
    pub fn create_test_proof(fragment_id: u32) -> FragmentProof {
        let metadata = FragmentMetadata {
            fragment_id,
            constraint_count: 100,
            input_boundary_count: if fragment_id > 0 { 1 } else { 0 },
            output_boundary_count: if fragment_id < 3 { 1 } else { 0 },
            execution_position: fragment_id,
        };

        let exec_hash = ExecutionHashBuilder::new(fragment_id).build();

        let proof_output = FragmentProofOutput {
            previous_execution_hash: if fragment_id == 0 {
                ExecutionHash::genesis()
            } else {
                ExecutionHash::from_bytes([(fragment_id - 1) as u8; 32])
            },
            execution_hash: exec_hash,
            output_boundaries: vec![],
            public_outputs: vec![],
            metadata,
        };

        FragmentProof::new(metadata, vec![0; 256], proof_output)
    }

    /// Create test metadata
    pub fn create_test_metadata(fragment_id: u32) -> FragmentMetadata {
        FragmentMetadata {
            fragment_id,
            constraint_count: 100,
            input_boundary_count: if fragment_id > 0 { 1 } else { 0 },
            output_boundary_count: if fragment_id < 3 { 1 } else { 0 },
            execution_position: fragment_id,
        }
    }

    /// Create test witness
    pub fn create_test_witness() -> FragmentWitness {
        FragmentWitness::new()
            .with_local_witness(vec![1, 2, 3, 4, 5])
            .with_public_inputs(vec![10, 20])
    }

    /// Create test capsule
    pub fn create_test_capsule(fragment_id: u32) -> FragmentProofCapsule {
        let metadata = create_test_metadata(fragment_id);
        let witness = create_test_witness();
        
        FragmentProofCapsule::new(metadata).with_witness(witness)
    }

    /// Create test commitment config
    pub fn create_test_commitment_config(fragment_id: u32) -> CommitmentConfig {
        CommitmentConfig::new(fragment_id)
    }

    /// Create test execution hash
    pub fn create_test_execution_hash(fragment_id: u32) -> ExecutionHash {
        ExecutionHashBuilder::new(fragment_id).build()
    }

    #[cfg(test)]
    mod utility_tests {
        use super::*;

        #[test]
        fn test_create_test_proof() {
            let proof = create_test_proof(0);
            assert_eq!(proof.metadata.fragment_id, 0);
        }

        #[test]
        fn test_create_test_metadata() {
            let metadata = create_test_metadata(1);
            assert_eq!(metadata.fragment_id, 1);
        }

        #[test]
        fn test_create_test_witness() {
            let witness = create_test_witness();
            assert_eq!(witness.local_witness.len(), 5);
        }

        #[test]
        fn test_create_test_capsule() {
            let capsule = create_test_capsule(2);
            assert_eq!(capsule.metadata.fragment_id, 2);
            assert!(!capsule.is_proven);
        }
    }
}

