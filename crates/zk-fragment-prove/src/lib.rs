//! ZK-FRAGMENT Week 3: Fragment Proving System
//!
//! This crate provides the fragment proving infrastructure including:
//! - Poseidon-based commitment schemes
//! - Execution hash chain tracking
//! - Fragment Proof Capsule construction
//! - Boundary verification

pub mod commitment;
pub mod execution_hash;
pub mod capsule;

pub use commitment::{
    Commitment, CommitmentOpening, BoundaryCommitment,
    CommitmentGenerator, CommitmentConfig, CommitmentVerifier,
};

pub use execution_hash::{ExecutionHash, ExecutionHashBuilder, ChainVerifier};

pub use capsule::{
    FragmentProofCapsule, FragmentProof, FragmentWitness,
    FragmentMetadata, FragmentProofOutput,
};

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_fragment_workflow() {
        // Step 1: Create commitment generator
        let gen_config = CommitmentConfig::new(1);
        let gen = CommitmentGenerator::new(gen_config);
        
        // Step 2: Generate boundary commitments
        let boundary_value = 42u64;
        let boundary = gen.generate_commitment(100, boundary_value);
        
        // Step 3: Create execution hash
        let exec_hash = ExecutionHashBuilder::new(1).build();
        
        // Step 4: Create capsule metadata
        let metadata = capsule::FragmentMetadata {
            fragment_id: 1,
            constraint_count: 100,
            input_boundary_count: 0,
            output_boundary_count: 1,
            execution_position: 0,
        };
        
        // Step 5: Create capsule
        let capsule = FragmentProofCapsule::new(metadata);
        
        // Verify all pieces exist
        assert!(!capsule.is_proven);
        assert_ne!(exec_hash.value, [0u8; 32]);
    }
}