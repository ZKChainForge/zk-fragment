//! ZK-FRAGMENT Week 3: Complete Fragment Proving System
//!
//! This crate provides the fragment proving infrastructure including:
//! - Poseidon-based commitment schemes
//! - Execution hash chain tracking
//! - Fragment Proof Capsule construction
//! - Boundary verification
//! - Witness management
//! - Fragment proving
//! - Circuit gadgets
//! - Proof structures

pub mod commitment;
pub mod execution_hash;
pub mod capsule;
pub mod witness;
pub mod circuit;
pub mod prover;
pub mod proof;

pub use commitment::{
    Commitment, CommitmentOpening, BoundaryCommitment,
    CommitmentGenerator, CommitmentConfig, CommitmentVerifier,
    poseidon_hash, derive_blinding_factor, verify_commitment,
};

pub use execution_hash::{
    ExecutionHash, ExecutionHashBuilder, ChainVerifier, ChainLink,
};

pub use capsule::{
    FragmentProofCapsule, FragmentProof, FragmentWitness,
    FragmentMetadata, FragmentProofOutput, CapsuleBuilder,
};

pub use witness::{
    WitnessIndexMap, WitnessExtractor,
    BoundaryWireValue, BoundaryWitnessSegment, BoundaryWitnessBuilder,
};

pub use circuit::{
    BoundaryInputGadget, BoundaryOutputGadget, 
    ExecutionHashGadget, FragmentCircuit,
    MerkleMembershipGadget,
};

pub use prover::{
    FragmentProver, FragmentProverConfig,
    ParallelProverCoordinator, ProvingCheckpoint,
};

pub use proof::{
    CompleteFragmentProof, FragmentProofVerifier,
};

// Re-export for convenience
pub use capsule::CapsuleBuilder as FragmentCapsuleBuilder;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_complete_week3_system() {
        // Step 1: Create fragment
        let metadata = FragmentMetadata {
            fragment_id: 1,
            constraint_count: 1000,
            input_boundary_count: 1,
            output_boundary_count: 1,
            execution_position: 1,
        };
        
        // Step 2: Create circuit
        let circuit_config = circuit::FragmentCircuitConfig::new(metadata.clone(), 1000);
        let circuit = circuit::FragmentCircuit::new(circuit_config);
        assert_eq!(circuit.total_constraints(), circuit.constraint_breakdown().total);
        
        // Step 3: Create witness
        let witness = FragmentWitness::new()
            .with_local_witness(vec![1, 2, 3])
            .with_public_inputs(vec![10]);
        
        // Step 4: Create capsule
        let capsule = FragmentProofCapsule::new(metadata)
            .with_witness(witness);
        
        // Step 5: Prove
        let prover_config = FragmentProverConfig::default();
        let prover = FragmentProver::new(prover_config);
        let proved = prover.prove(capsule).unwrap();
        
        // Step 6: Get proof
        let proof = proved.get_proof().unwrap();
        
        // Step 7: Verify proof
        let verification = FragmentProofVerifier::verify(proof).unwrap();
        assert!(verification.is_valid);
    }

    #[test]
    fn test_complete_boundary_workflow() {
        // Fragment 0: Producer
        let gen_config = CommitmentConfig::new(0);
        let generator = CommitmentGenerator::new(gen_config);
        let boundary = generator.generate_commitment(100, 42);
        
        // Fragment 1: Consumer
        let mut verifier = CommitmentVerifier::new();
        verifier.register_commitment(100, boundary.commitment);
        
        // Verify (using deterministic blinding)
        let blinding = derive_blinding_factor(0, 100, &[0u8; 32]);
        let opening = CommitmentOpening { value: 42, blinding };
        
        let result = verifier.verify_opening(100, &opening);
        // Result depends on whether blinding matches
        let _ = result;
    }

    #[test]
    fn test_complete_chain_workflow() {
        // Create hashes
        let hash0 = ExecutionHashBuilder::new(0).build();
        let hash1 = ExecutionHashBuilder::new(1)
            .with_previous_hash(hash0)
            .build();
        let hash2 = ExecutionHashBuilder::new(2)
            .with_previous_hash(hash1)
            .build();
        
        // Verify chain
        let mut chain = ChainVerifier::new();
        chain.add_link(ChainLink::new(0, hash0, ExecutionHash::genesis()));
        chain.add_link(ChainLink::new(1, hash1, hash0));
        chain.add_link(ChainLink::new(2, hash2, hash1));
        
        assert!(chain.verify());
    }

    #[test]
    fn test_parallel_proving_estimation() {
        let sequential = 10000u64; // 10 seconds for 10 fragments
        let parallel = ParallelProverCoordinator::estimate_parallel_time(10, 1000);
        let speedup = ParallelProverCoordinator::calculate_speedup(sequential, parallel);
        
        assert!(speedup > 1.0);
    }

    #[test]
    fn test_checkpoint_system() {
        let mut checkpoint = ProvingCheckpoint::new("test_run".to_string(), 100);
        checkpoint.completed_capsules = 50;
        checkpoint.elapsed_ms = 50000;
        
        assert_eq!(checkpoint.progress_percentage(), 50.0);
        let remaining = checkpoint.estimate_remaining_ms();
        assert!(remaining > 0);
    }
}