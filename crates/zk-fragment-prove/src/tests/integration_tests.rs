//! Integration tests for complete Week 3 workflows

#[cfg(test)]
mod complete_fragment_workflow {
    use zk_fragment_prove::*;

    #[test]
    fn test_single_fragment_complete_workflow() {
        // Step 1: Create metadata
        let metadata = FragmentMetadata {
            fragment_id: 0,
            constraint_count: 100,
            input_boundary_count: 0,
            output_boundary_count: 1,
            execution_position: 0,
        };
        
        // Step 2: Create circuit
        let circuit_config = circuit::FragmentCircuitConfig::new(metadata.clone(), 100);
        let circuit = circuit::FragmentCircuit::new(circuit_config);
        
        // Step 3: Verify circuit constraints
        let breakdown = circuit.constraint_breakdown();
        assert_eq!(breakdown.fragment_constraints, 100);
        assert!(breakdown.total > 100);
        
        // Step 4: Create witness
        let witness = FragmentWitness::new()
            .with_local_witness(vec![1, 2, 3, 4, 5])
            .with_public_inputs(vec![10]);
        
        assert_eq!(witness.local_witness.len(), 5);
        
        // Step 5: Create capsule
        let capsule = FragmentProofCapsule::new(metadata.clone())
            .with_witness(witness);
        
        assert!(!capsule.is_proven);
        
        // Step 6: Prove
        let prover_config = FragmentProverConfig::default();
        let prover = FragmentProver::new(prover_config);
        let proved = prover.prove(capsule).unwrap();
        
        assert!(proved.is_proven);
        assert!(proved.get_proof().is_some());
        
        // Step 7: Verify proof
        let proof = proved.get_proof().unwrap();
        assert_eq!(proof.metadata.fragment_id, 0);
    }

    #[test]
    fn test_two_fragment_chain_workflow() {
        // Create Fragment 0
        let metadata_0 = FragmentMetadata {
            fragment_id: 0,
            constraint_count: 100,
            input_boundary_count: 0,
            output_boundary_count: 1,
            execution_position: 0,
        };
        
        let exec_hash_0 = ExecutionHashBuilder::new(0).build();
        let boundary_0 = {
            let gen = CommitmentGenerator::new(CommitmentConfig::new(0));
            gen.generate_commitment(100, 42)
        };
        
        let proof_output_0 = FragmentProofOutput {
            previous_execution_hash: ExecutionHash::genesis(),
            execution_hash: exec_hash_0,
            output_boundaries: vec![boundary_0.clone()],
            public_outputs: vec![],
            metadata: metadata_0.clone(),
        };
        
        let proof_0 = FragmentProof::new(metadata_0, vec![0; 256], proof_output_0)
            .with_constraint_count(100);
        
        // Create Fragment 1
        let metadata_1 = FragmentMetadata {
            fragment_id: 1,
            constraint_count: 80,
            input_boundary_count: 1,
            output_boundary_count: 0,
            execution_position: 1,
        };
        
        let exec_hash_1 = ExecutionHashBuilder::new(1)
            .with_previous_hash(exec_hash_0)
            .build();
        
        let proof_output_1 = FragmentProofOutput {
            previous_execution_hash: exec_hash_0,
            execution_hash: exec_hash_1,
            output_boundaries: vec![],
            public_outputs: vec![100],
            metadata: metadata_1.clone(),
        };
        
        let proof_1 = FragmentProof::new(metadata_1, vec![0; 256], proof_output_1)
            .with_constraint_count(80);
        
        // Verify chain
        let mut chain = ChainVerifier::new();
        chain.add_link(ChainLink::new(0, exec_hash_0, ExecutionHash::genesis()));
        chain.add_link(ChainLink::new(1, exec_hash_1, exec_hash_0));
        
        assert!(chain.verify());
        assert_eq!(chain.final_hash(), Some(exec_hash_1));
    }

    #[test]
    fn test_four_fragment_sequence() {
        let mut proofs = Vec::new();
        let mut hashes = Vec::new();
        
        for frag_id in 0..4 {
            let metadata = FragmentMetadata {
                fragment_id: frag_id,
                constraint_count: 100,
                input_boundary_count: if frag_id > 0 { 1 } else { 0 },
                output_boundary_count: if frag_id < 3 { 1 } else { 0 },
                execution_position: frag_id,
            };
            
            let exec_hash = ExecutionHashBuilder::new(frag_id).build();
            hashes.push(exec_hash);
            
            let prev_hash = if frag_id == 0 {
                ExecutionHash::genesis()
            } else {
                hashes[frag_id as usize - 1]
            };
            
            let proof_output = FragmentProofOutput {
                previous_execution_hash: prev_hash,
                execution_hash: exec_hash,
                output_boundaries: vec![],
                public_outputs: if frag_id == 3 { vec![100] } else { vec![] },
                metadata,
            };
            
            let proof = FragmentProof::new(metadata, vec![0; 256], proof_output)
                .with_constraint_count(100);
            
            proofs.push(proof);
        }
        
        // Verify all proofs are present
        assert_eq!(proofs.len(), 4);
        
        // Verify hashes are all different
        let mut unique_hashes = std::collections::HashSet::new();
        for hash in &hashes {
            unique_hashes.insert(hash.value);
        }
        assert_eq!(unique_hashes.len(), 4);
    }
}

#[cfg(test)]
mod witness_management_integration {
    use zk_fragment_prove::*;

    #[test]
    fn test_witness_extraction_complete() {
        let mut index_map = WitnessIndexMap::new();
        
        // Map indices for a 3-constraint fragment
        index_map.add_mapping(0, 0);  // Full index 0 -> Fragment index 0
        index_map.add_mapping(3, 1);  // Full index 3 -> Fragment index 1
        index_map.add_mapping(5, 2);  // Full index 5 -> Fragment index 2
        
        let full_witness = vec![10, 20, 30, 40, 50, 60];
        
        let extractor = WitnessExtractor::new(index_map);
        let fragment_witness = extractor.extract(&full_witness).unwrap();
        
        assert_eq!(fragment_witness.len(), 3);
        assert_eq!(fragment_witness[0], 10);
        assert_eq!(fragment_witness[1], 40);
        assert_eq!(fragment_witness[2], 60);
    }

    #[test]
    fn test_boundary_witness_workflow() {
        let builder = BoundaryWitnessBuilder::new(1, [1u8; 32]);
        
        let inputs = builder.create_input_boundaries(&[(100, 42), (101, 43)]);
        let outputs = builder.create_output_boundaries(&[(102, 44)]);
        
        let segment = BoundaryWitnessSegment::new()
            .with_input_boundaries(inputs)
            .with_output_boundaries(outputs);
        
        assert_eq!(segment.input_boundaries.len(), 2);
        assert_eq!(segment.output_boundaries.len(), 1);
    }

    #[test]
    fn test_fragment_witness_construction() {
        let mut boundary_segment = BoundaryWitnessSegment::new();
        
        let boundary = BoundaryWireValue::new(100, 42, [1u8; 32]);
        boundary_segment = boundary_segment.with_input_boundary(boundary);
        
        let witness = FragmentWitness::new()
            .with_local_witness(vec![1, 2, 3])
            .with_public_inputs(vec![10])
            .with_input_boundaries(boundary_segment.input_boundaries);
        
        assert_eq!(witness.local_witness.len(), 3);
        assert_eq!(witness.public_inputs.len(), 1);
        assert_eq!(witness.input_boundaries.len(), 1);
    }

    #[test]
    fn test_complex_witness_partitioning() {
        // Simulate a circuit with 20 wires, fragmenting into 2 parts
        let full_witness: Vec<u64> = (0..20).collect();
        
        // Fragment 0 needs wires: 0, 2, 4, 6
        let mut map_0 = WitnessIndexMap::new();
        for i in [0, 2, 4, 6].iter() {
            map_0.add_mapping(*i, *i / 2);
        }
        
        let extractor_0 = WitnessExtractor::new(map_0);
        let witness_0 = extractor_0.extract(&full_witness).unwrap();
        
        assert_eq!(witness_0.len(), 4);
        assert_eq!(witness_0[0], 0);
        assert_eq!(witness_0[1], 2);
        assert_eq!(witness_0[2], 4);
        assert_eq!(witness_0[3], 6);
        
        // Fragment 1 needs wires: 1, 3, 5, 7
        let mut map_1 = WitnessIndexMap::new();
        for i in [1, 3, 5, 7].iter() {
            map_1.add_mapping(*i, *i / 2);
        }
        
        let extractor_1 = WitnessExtractor::new(map_1);
        let witness_1 = extractor_1.extract(&full_witness).unwrap();
        
        assert_eq!(witness_1.len(), 4);
    }
}

#[cfg(test)]
mod execution_hash_integration {
    use zk_fragment_prove::*;

    #[test]
    fn test_execution_hash_chain_complete() {
        let mut chain = ChainVerifier::new();
        
        // Create a chain of 5 hashes
        let mut prev_hash = ExecutionHash::genesis();
        
        for frag_id in 0..5 {
            let hash = ExecutionHashBuilder::new(frag_id)
                .with_previous_hash(prev_hash)
                .build();
            
            chain.add_link(ChainLink::new(frag_id, hash, prev_hash));
            prev_hash = hash;
        }
        
        // Verify chain
        assert!(chain.verify());
        assert!(chain.final_hash().is_some());
    }

    #[test]
    fn test_execution_hash_with_boundaries() {
        let gen = CommitmentGenerator::new(CommitmentConfig::new(0));
        
        // Generate boundary commitments
        let boundaries = gen.generate_commitments(&[(100, 42), (101, 43)]);
        
        // Use commitments in hash
        let mut hash_builder = ExecutionHashBuilder::new(0);
        for boundary in &boundaries {
            hash_builder = hash_builder
                .with_commitment(boundary.commitment.value.to_le_bytes());
        }
        
        let exec_hash = hash_builder.build();
        assert_ne!(exec_hash.value, [0u8; 32]);
    }
}

#[cfg(test)]
mod prover_integration {
    use zk_fragment_prove::*;

    #[test]
    fn test_fragment_prover_complete_workflow() {
        let metadata = FragmentMetadata {
            fragment_id: 0,
            constraint_count: 100,
            input_boundary_count: 0,
            output_boundary_count: 1,
            execution_position: 0,
        };
        
        let witness = FragmentWitness::new()
            .with_local_witness(vec![1, 2, 3])
            .with_public_inputs(vec![10]);
        
        let capsule = FragmentProofCapsule::new(metadata)
            .with_witness(witness);
        
        let config = FragmentProverConfig::default();
        let prover = FragmentProver::new(config);
        
        let proved = prover.prove(capsule).unwrap();
        
        assert!(proved.is_proven);
        assert!(proved.get_proof().is_some());
        assert!(proved.get_execution_hash().is_some());
    }

    #[test]
    fn test_parallel_prover_estimation() {
        let sequential_time = 1000u64; // 1 second per fragment
        let parallel_estimate = ParallelProverCoordinator::estimate_parallel_time(8, 1000);
        
        assert!(parallel_estimate > 0);
        assert!(parallel_estimate < sequential_time * 8);
    }

    #[test]
    fn test_checkpoint_tracking() {
        let mut checkpoint = ProvingCheckpoint::new("test".to_string(), 100);
        
        checkpoint.completed_capsules = 50;
        checkpoint.elapsed_ms = 5000;
        
        assert_eq!(checkpoint.progress_percentage(), 50.0);
        
        let remaining = checkpoint.estimate_remaining_ms();
        assert!(remaining > 0);
    }
}

#[cfg(test)]
mod circuit_integration {
    use zk_fragment_prove::*;

    #[test]
    fn test_fragment_circuit_complete() {
        let metadata = FragmentMetadata {
            fragment_id: 0,
            constraint_count: 500,
            input_boundary_count: 1,
            output_boundary_count: 1,
            execution_position: 0,
        };
        
        let config = circuit::FragmentCircuitConfig::new(metadata, 500);
        let circuit = circuit::FragmentCircuit::new(config);
        
        let breakdown = circuit.constraint_breakdown();
        
        assert_eq!(breakdown.fragment_constraints, 500);
        assert!(breakdown.total > 500);
        
        let overhead = breakdown.boundary_overhead();
        assert!(overhead > 0.0);
        assert!(overhead < 100.0);
    }
}

#[cfg(test)]
mod complete_system_test {
    use zk_fragment_prove::*;

    #[test]
    fn test_complete_week3_system() {
        // Create metadata
        let metadata = FragmentMetadata {
            fragment_id: 1,
            constraint_count: 1000,
            input_boundary_count: 1,
            output_boundary_count: 1,
            execution_position: 1,
        };
        
        // Create circuit
        let circuit_config = circuit::FragmentCircuitConfig::new(metadata.clone(), 1000);
        let circuit = circuit::FragmentCircuit::new(circuit_config);
        assert!(circuit.total_constraints() > 1000);
        
        // Create commitment generator
        let gen = CommitmentGenerator::new(CommitmentConfig::new(1));
        let boundary = gen.generate_commitment(100, 42);
        assert_ne!(boundary.commitment.value, 0);
        
        // Create execution hash
        let exec_hash = ExecutionHashBuilder::new(1).build();
        assert_ne!(exec_hash.value, [0u8; 32]);
        
        // Create witness
        let witness = FragmentWitness::new()
            .with_local_witness(vec![1, 2, 3])
            .with_public_inputs(vec![10]);
        assert_eq!(witness.local_witness.len(), 3);
        
        // Create capsule
        let capsule = FragmentProofCapsule::new(metadata)
            .with_witness(witness);
        assert!(!capsule.is_proven);
        
        // Prove
        let prover = FragmentProver::new(FragmentProverConfig::default());
        let proved = prover.prove(capsule).unwrap();
        assert!(proved.is_proven);
    }
}