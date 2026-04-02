//! Integration tests for complete workflows

#[cfg(test)]
mod tests {
    use zk_fragment_prove::*;

    #[test]
    fn test_single_fragment_workflow() {
        // Setup
        let metadata = FragmentMetadata {
            fragment_id: 1,
            constraint_count: 100,
            input_boundary_count: 0,
            output_boundary_count: 1,
            execution_position: 0,
        };
        
        // Generate boundary
        let gen_config = CommitmentConfig::new(metadata.fragment_id);
        let gen = CommitmentGenerator::new(gen_config);
        let boundary = gen.generate_commitment(100, 42u64);
        
        // Create execution hash
        let exec_hash = ExecutionHashBuilder::new(metadata.fragment_id)
            .build();
        
        // Create proof output
        let proof_output = FragmentProofOutput {
            previous_execution_hash: ExecutionHash::genesis(),
            execution_hash: exec_hash,
            output_boundaries: vec![boundary],
            public_outputs: vec![],
            metadata: metadata.clone(),
        };
        
        // Create proof
        let proof = FragmentProof::new(
            metadata.clone(),
            vec![0; 256],
            proof_output,
        );
        
        // Create capsule
        let mut capsule = FragmentProofCapsule::new(metadata.clone())
            .with_witness(FragmentWitness::new());
        capsule.set_proof(proof);
        
        // Verify
        assert!(capsule.is_proven);
        assert!(capsule.get_proof().is_some());
        assert!(capsule.get_execution_hash().is_some());
    }

    #[test]
    fn test_two_fragment_boundary_chain() {
        // Fragment 0
        let metadata_0 = FragmentMetadata {
            fragment_id: 0,
            constraint_count: 100,
            input_boundary_count: 0,
            output_boundary_count: 1,
            execution_position: 0,
        };
        
        let gen_0 = CommitmentGenerator::new(CommitmentConfig::new(0));
        let boundary_0 = gen_0.generate_commitment(100, 42u64);
        
        let exec_hash_0 = ExecutionHashBuilder::new(0).build();
        
        let proof_output_0 = FragmentProofOutput {
            previous_execution_hash: ExecutionHash::genesis(),
            execution_hash: exec_hash_0,
            output_boundaries: vec![boundary_0.clone()],
            public_outputs: vec![],
            metadata: metadata_0.clone(),
        };
        
        let proof_0 = FragmentProof::new(metadata_0, vec![0; 256], proof_output_0);
        
        // Fragment 1
        let metadata_1 = FragmentMetadata {
            fragment_id: 1,
            constraint_count: 80,
            input_boundary_count: 1,
            output_boundary_count: 0,
            execution_position: 1,
        };
        
        let opening_1 = CommitmentOpening {
            value: 42u64,
            blinding: [1u8; 32],
        };
        
        let exec_hash_1 = ExecutionHashBuilder::new(1)
            .with_previous_hash(exec_hash_0)
            .build();
        
        let proof_output_1 = FragmentProofOutput {
            previous_execution_hash: exec_hash_0,
            execution_hash: exec_hash_1,
            output_boundaries: vec![],
            public_outputs: vec![],
            metadata: metadata_1.clone(),
        };
        
        let proof_1 = FragmentProof::new(metadata_1, vec![0; 256], proof_output_1);
        
        // Verify chain
        let mut chain = execution_hash::ChainVerifier::new();
        chain.add_link(execution_hash::ChainLink::new(0, exec_hash_0, ExecutionHash::genesis()));
        chain.add_link(execution_hash::ChainLink::new(1, exec_hash_1, exec_hash_0));
        
        assert!(chain.verify());
        assert_eq!(chain.final_hash(), Some(exec_hash_1));
    }

    #[test]
    fn test_boundary_mismatch_detection() {
        let blinding = [1u8; 32];
        let commitment = poseidon_hash(42u64, &blinding);
        
        // Producer produces 42
        let producer_value = 42u64;
        
        // Consumer receives different value
        let consumer_value = 43u64;
        let consumer_opening = CommitmentOpening {
            value: consumer_value,
            blinding,
        };
        
        // Verification should fail
        assert!(!verify_commitment(
            &Commitment { value: commitment.value },
            &consumer_opening
        ));
    }
}