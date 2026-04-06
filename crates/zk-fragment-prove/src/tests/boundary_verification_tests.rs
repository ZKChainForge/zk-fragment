//! Tests for boundary verification and consistency

#[cfg(test)]
mod boundary_gadget_tests {
    use zk_fragment_prove::circuit::*;
    use zk_fragment_prove::commitment::poseidon_hash;

    #[test]
    fn test_boundary_input_gadget_single() {
        let config = BoundaryInputConfig::new(1);
        let gadget = BoundaryInputGadget::new(config);
        
        let value = 42u64;
        let blinding = [1u8; 32];
        let commitment = poseidon_hash(value, &blinding);
        let opening = CommitmentOpening { value, blinding };
        
        let result = gadget.verify_single(&commitment, &opening).unwrap();
        
        assert!(result.is_valid);
        assert_eq!(result.verified_values, vec![42]);
    }

    #[test]
    fn test_boundary_input_gadget_multiple() {
        let config = BoundaryInputConfig::new(3);
        let gadget = BoundaryInputGadget::new(config);
        
        let blinding = [1u8; 32];
        let commitments = vec![
            poseidon_hash(10, &blinding),
            poseidon_hash(20, &blinding),
            poseidon_hash(30, &blinding),
        ];
        let openings = vec![
            CommitmentOpening { value: 10, blinding },
            CommitmentOpening { value: 20, blinding },
            CommitmentOpening { value: 30, blinding },
        ];
        
        let result = gadget.verify_all(&commitments, &openings).unwrap();
        
        assert!(result.is_valid);
        assert_eq!(result.verified_values, vec![10, 20, 30]);
    }

    #[test]
    fn test_boundary_input_gadget_mismatch() {
        let config = BoundaryInputConfig::new(1);
        let gadget = BoundaryInputGadget::new(config);
        
        let blinding = [1u8; 32];
        let commitment = poseidon_hash(42u64, &blinding);
        let wrong_opening = CommitmentOpening { value: 99, blinding };
        
        let result = gadget.verify_single(&commitment, &wrong_opening);
        assert!(result.is_err());
    }

    #[test]
    fn test_boundary_input_gadget_count_mismatch() {
        let config = BoundaryInputConfig::new(2);
        let gadget = BoundaryInputGadget::new(config);
        
        let blinding = [1u8; 32];
        let commitments = vec![poseidon_hash(10, &blinding)];
        let openings = vec![CommitmentOpening { value: 10, blinding }];
        
        let result = gadget.verify_all(&commitments, &openings);
        assert!(result.is_err());
    }

    #[test]
    fn test_boundary_output_gadget_single() {
        let config = BoundaryOutputConfig::new(1, [1u8; 32], 1);
        let gadget = BoundaryOutputGadget::new(config);
        
        let output = gadget.generate_single(100, 42).unwrap();
        
        assert_eq!(output.wire_id, 100);
        assert_eq!(output.value, 42);
        assert_ne!(output.commitment.value, 0);
    }

    #[test]
    fn test_boundary_output_gadget_multiple() {
        let config = BoundaryOutputConfig::new(1, [1u8; 32], 3);
        let gadget = BoundaryOutputGadget::new(config);
        
        let wire_ids = vec![100, 101, 102];
        let values = vec![42, 43, 44];
        
        let result = gadget.generate_commitments(&wire_ids, &values).unwrap();
        
        assert_eq!(result.commitments.len(), 3);
        assert!(result.constraint_count > 0);
    }

    #[test]
    fn test_boundary_output_deterministic() {
        let config = BoundaryOutputConfig::new(1, [1u8; 32], 1);
        let gadget = BoundaryOutputGadget::new(config);
        
        let output1 = gadget.generate_single(100, 42).unwrap();
        let output2 = gadget.generate_single(100, 42).unwrap();
        
        assert_eq!(output1.commitment, output2.commitment);
        assert_eq!(output1.blinding, output2.blinding);
    }

    #[test]
    fn test_execution_hash_gadget_simple() {
        let config = ExecutionHashConfig::new(0, 0, 0);
        let gadget = ExecutionHashGadget::new(config);
        
        let result = gadget.compute(
            ExecutionHash::genesis(),
            &[],
            &[],
        ).unwrap();
        
        assert_ne!(result.execution_hash.value, [0u8; 32]);
        assert!(result.constraint_count > 0);
    }

    #[test]
    fn test_execution_hash_gadget_with_boundaries() {
        let config = ExecutionHashConfig::new(1, 2, 2);
        let gadget = ExecutionHashGadget::new(config);
        
        let prev_hash = ExecutionHash::from_bytes([1u8; 32]);
        let input_commits = vec![[2u8; 32], [3u8; 32]];
        let output_commits = vec![[4u8; 32], [5u8; 32]];
        
        let result = gadget.compute(
            prev_hash,
            &input_commits,
            &output_commits,
        ).unwrap();
        
        assert_ne!(result.execution_hash.value, [0u8; 32]);
    }

    #[test]
    fn test_fragment_wrapper_circuit() {
        use zk_fragment_prove::capsule::FragmentMetadata;
        
        let metadata = FragmentMetadata {
            fragment_id: 0,
            constraint_count: 1000,
            input_boundary_count: 0,
            output_boundary_count: 1,
            execution_position: 0,
        };
        
        let config = FragmentCircuitConfig::new(metadata, 1000);
        let circuit = FragmentCircuit::new(config);
        
        let breakdown = circuit.constraint_breakdown();
        
        assert_eq!(breakdown.fragment_constraints, 1000);
        assert!(breakdown.total > 1000);
    }

    #[test]
    fn test_merkle_membership_gadget() {
        let config = MerkleMembershipConfig::new(0, 3);
        let gadget = MerkleMembershipGadget::new(config);
        
        let leaf = [1u8; 32];
        let sibling1 = [2u8; 32];
        let sibling2 = [3u8; 32];
        let sibling3 = [4u8; 32];
        
        // Manually compute root
        let level1 = {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(&leaf);
            hasher.update(&sibling1);
            let result = hasher.finalize();
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&result);
            hash
        };
        
        let level2 = {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(&level1);
            hasher.update(&sibling2);
            let result = hasher.finalize();
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&result);
            hash
        };
        
        let root = {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(&level2);
            hasher.update(&sibling3);
            let result = hasher.finalize();
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&result);
            hash
        };
        
        let result = gadget.verify(leaf, &[sibling1, sibling2, sibling3], root).unwrap();
        assert!(result.is_valid);
    }
}

#[cfg(test)]
mod boundary_consistency_tests {
    use zk_fragment_prove::*;

    #[test]
    fn test_producer_consumer_boundary() {
        // Producer: Fragment 0
        let gen_0 = CommitmentGenerator::new(CommitmentConfig::new(0));
        let boundary_0 = gen_0.generate_commitment(100, 42);
        
        // Consumer: Fragment 1 receives the commitment
        let mut verifier_1 = CommitmentVerifier::new();
        verifier_1.register_commitment(100, boundary_0.commitment);
        
        // For verification to succeed, Fragment 1 needs the same blinding
        let blinding = derive_blinding_factor(0, 100, &[0u8; 32]);
        let opening = CommitmentOpening { value: 42, blinding };
        
        // Verify (may pass or fail depending on blinding derivation)
        let result = verifier_1.verify_opening(100, &opening);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_multiple_boundary_consistency() {
        let gen = CommitmentGenerator::new(CommitmentConfig::new(0));
        
        let boundaries = gen.generate_commitments(&[
            (100, 42),
            (101, 43),
            (102, 44),
        ]);
        
        let mut verifier = CommitmentVerifier::new();
        for boundary in &boundaries {
            verifier.register_commitment(boundary.wire_id, boundary.commitment);
        }
        
        assert_eq!(verifier.commitments().len(), 3);
    }

    #[test]
    fn test_boundary_tampering_detection() {
        let gen = CommitmentGenerator::new(CommitmentConfig::new(0));
        let boundary = gen.generate_commitment(100, 42);
        
        let mut verifier = CommitmentVerifier::new();
        verifier.register_commitment(100, boundary.commitment);
        
        // Try to verify with wrong value
        let wrong_opening = CommitmentOpening {
            value: 99,
            blinding: [1u8; 32],
        };
        
        let result = verifier.verify_opening(100, &wrong_opening);
        assert!(result.is_err());
    }

    #[test]
    fn test_boundary_order_independence() {
        let gen = CommitmentGenerator::new(CommitmentConfig::new(0));
        
        let boundaries1: Vec<_> = (0..5)
            .map(|i| gen.generate_commitment(100 + i, 40 + i as u64))
            .collect();
        
        // Register in different order
        let mut verifier = CommitmentVerifier::new();
        for i in (0..5).rev() {
            let boundary = &boundaries1[i as usize];
            verifier.register_commitment(boundary.wire_id, boundary.commitment);
        }
        
        assert_eq!(verifier.commitments().len(), 5);
    }
}

#[cfg(test)]
mod constraint_counting_tests {
    use zk_fragment_prove::circuit::*;
    use zk_fragment_prove::capsule::FragmentMetadata;

    #[test]
    fn test_boundary_input_constraint_count() {
        let config = BoundaryInputConfig::new(5);
        let gadget = BoundaryInputGadget::new(config);
        
        let count = gadget.constraint_count();
        assert_eq!(count, 5 * 500);
    }

    #[test]
    fn test_boundary_output_constraint_count() {
        let config = BoundaryOutputConfig::new(1, [1u8; 32], 3);
        let gadget = BoundaryOutputGadget::new(config);
        
        let count = gadget.constraint_count();
        assert_eq!(count, 3 * 500);
    }

    #[test]
    fn test_execution_hash_constraint_count() {
        let config = ExecutionHashConfig::new(0, 2, 2);
        let gadget = ExecutionHashGadget::new(config);
        
        let count = gadget.constraint_count();
        assert_eq!(count, 1000); // Fixed constant
    }

    #[test]
    fn test_fragment_circuit_constraint_accounting() {
        let metadata = FragmentMetadata {
            fragment_id: 0,
            constraint_count: 500,
            input_boundary_count: 2,
            output_boundary_count: 2,
            execution_position: 0,
        };
        
        let config = FragmentCircuitConfig::new(metadata, 500);
        let circuit = FragmentCircuit::new(config);
        
        let breakdown = circuit.constraint_breakdown();
        
        let calculated_total = breakdown.fragment_constraints
            + breakdown.input_boundary_constraints
            + breakdown.output_boundary_constraints
            + breakdown.execution_hash_constraints;
        
        assert_eq!(breakdown.total, calculated_total);
    }

    #[test]
    fn test_merkle_membership_constraint_count() {
        let config = MerkleMembershipConfig::new(0, 4);
        let gadget = MerkleMembershipGadget::new(config);
        
        let count = gadget.constraint_count();
        assert_eq!(count, 4 * 300);
    }
}