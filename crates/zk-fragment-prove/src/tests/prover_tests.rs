//! Tests for fragment proving system

#[cfg(test)]
mod fragment_prover_tests {
    use zk_fragment_prove::*;

    #[test]
    fn test_fragment_prover_creation() {
        let config = FragmentProverConfig::default();
        let _prover = FragmentProver::new(config);
    }

    #[test]
    fn test_fragment_prover_simple_proof() {
        let config = FragmentProverConfig::default();
        let prover = FragmentProver::new(config);
        
        let metadata = FragmentMetadata {
            fragment_id: 0,
            constraint_count: 100,
            input_boundary_count: 0,
            output_boundary_count: 1,
            execution_position: 0,
        };
        
        let capsule = FragmentProofCapsule::new(metadata);
        let proved = prover.prove(capsule).unwrap();
        
        assert!(proved.is_proven);
        assert_eq!(proved.metadata.fragment_id, 0);
    }

    #[test]
    fn test_fragment_prover_with_witness() {
        let config = FragmentProverConfig::default();
        let prover = FragmentProver::new(config);
        
        let metadata = FragmentMetadata {
            fragment_id: 0,
            constraint_count: 100,
            input_boundary_count: 0,
            output_boundary_count: 0,
            execution_position: 0,
        };
        
        let witness = FragmentWitness::new()
            .with_local_witness(vec![1, 2, 3])
            .with_public_inputs(vec![10]);
        
        let capsule = FragmentProofCapsule::new(metadata)
            .with_witness(witness);
        
        let proved = prover.prove(capsule).unwrap();
        
        assert!(proved.is_proven);
    }

    #[test]
    fn test_fragment_prover_with_boundaries() {
        let config = FragmentProverConfig::default();
        let prover = FragmentProver::new(config);
        
        let metadata = FragmentMetadata {
            fragment_id: 1,
            constraint_count: 100,
            input_boundary_count: 1,
            output_boundary_count: 1,
            execution_position: 1,
        };
        
        let witness = FragmentWitness::new()
            .with_input_boundaries(vec![
                CommitmentOpening {
                    value: 42,
                    blinding: [1u8; 32],
                }
            ]);
        
        let capsule = FragmentProofCapsule::new(metadata)
            .with_witness(witness);
        
        let proved = prover.prove(capsule).unwrap();
        
        assert!(proved.is_proven);
        assert!(proved.get_output_boundaries().is_some());
    }

    #[test]
    fn test_fragment_prover_multiple_constraints() {
        let config = FragmentProverConfig::default();
        let prover = FragmentProver::new(config);
        
        let metadata = FragmentMetadata {
            fragment_id: 0,
            constraint_count: 500,
            input_boundary_count: 0,
            output_boundary_count: 0,
            execution_position: 0,
        };
        
        let witness = FragmentWitness::new()
            .with_local_witness(vec![1, 2, 3, 4, 5]);
        
        let capsule = FragmentProofCapsule::new(metadata)
            .with_witness(witness);
        
        let proved = prover.prove(capsule).unwrap();
        
        assert!(proved.is_proven);
        let proof = proved.get_proof().unwrap();
        assert_eq!(proof.proof_constraint_count, 500);
    }

    #[test]
    fn test_fragment_prover_execution_hash() {
        let config = FragmentProverConfig::default();
        let prover = FragmentProver::new(config);
        
        let metadata = FragmentMetadata {
            fragment_id: 0,
            constraint_count: 100,
            input_boundary_count: 0,
            output_boundary_count: 0,
            execution_position: 0,
        };
        
        let capsule = FragmentProofCapsule::new(metadata);
        let proved = prover.prove(capsule).unwrap();
        
        let exec_hash = proved.get_execution_hash().unwrap();
        assert_ne!(exec_hash.value, [0u8; 32]);
    }

    #[test]
    fn test_fragment_prover_timing() {
        let config = FragmentProverConfig::default();
        let prover = FragmentProver::new(config);
        
        let metadata = FragmentMetadata {
            fragment_id: 0,
            constraint_count: 100,
            input_boundary_count: 0,
            output_boundary_count: 0,
            execution_position: 0,
        };
        
        let capsule = FragmentProofCapsule::new(metadata);
        let proved = prover.prove(capsule).unwrap();
        
        let proof = proved.get_proof().unwrap();
        assert!(proof.proving_time_ms >= 0);
    }

    #[test]
    fn test_fragment_prover_already_proven() {
        let config = FragmentProverConfig::default();
        let prover = FragmentProver::new(config);
        
        let metadata = FragmentMetadata {
            fragment_id: 0,
            constraint_count: 100,
            input_boundary_count: 0,
            output_boundary_count: 0,
            execution_position: 0,
        };
        
        let capsule = FragmentProofCapsule::new(metadata.clone());
        let proved = prover.prove(capsule).unwrap();
        
        // Try to prove again
        let result = prover.prove(proved);
        assert!(result.is_err());
    }

    #[test]
    fn test_fragment_prover_multiple_fragments() {
        let config = FragmentProverConfig::default();
        let prover = FragmentProver::new(config);
        
        let capsules: Vec<_> = (0..4)
            .map(|i| {
                let metadata = FragmentMetadata {
                    fragment_id: i,
                    constraint_count: 100,
                    input_boundary_count: if i > 0 { 1 } else { 0 },
                    output_boundary_count: if i < 3 { 1 } else { 0 },
                    execution_position: i,
                };
                FragmentProofCapsule::new(metadata)
            })
            .collect();
        
        let proved_batch = prover.prove_batch(capsules).unwrap();
        
        assert_eq!(proved_batch.len(), 4);
        for (i, capsule) in proved_batch.iter().enumerate() {
            assert!(capsule.is_proven);
            assert_eq!(capsule.metadata.fragment_id, i as u32);
        }
    }

    #[test]
    fn test_fragment_prover_constraint_count_tracking() {
        let config = FragmentProverConfig::default();
        let prover = FragmentProver::new(config);
        
        for size in [100, 500, 1000, 2000].iter() {
            let metadata = FragmentMetadata {
                fragment_id: 0,
                constraint_count: *size,
                input_boundary_count: 0,
                output_boundary_count: 0,
                execution_position: 0,
            };
            
            let capsule = FragmentProofCapsule::new(metadata);
            let proved = prover.prove(capsule).unwrap();
            
            let proof = proved.get_proof().unwrap();
            assert_eq!(proof.proof_constraint_count, *size);
        }
    }
}

#[cfg(test)]
mod parallel_prover_tests {
    use zk_fragment_prove::*;

    #[test]
    fn test_parallel_prover_config_creation() {
        let config = ParallelProverConfig::default();
        assert!(config.num_threads > 0);
    }

    #[test]
    fn test_parallel_prover_coordinator_creation() {
        let config = ParallelProverConfig::default();
        let _coordinator = ParallelProverCoordinator::new(config);
    }

    #[test]
    fn test_parallel_time_estimation() {
        let time = ParallelProverCoordinator::estimate_parallel_time(4, 1000);
        assert!(time > 0);
        assert!(time < 4000);
    }

    #[test]
    fn test_parallel_time_scaling() {
        let time_1 = ParallelProverCoordinator::estimate_parallel_time(1, 1000);
        let time_2 = ParallelProverCoordinator::estimate_parallel_time(2, 1000);
        let time_4 = ParallelProverCoordinator::estimate_parallel_time(4, 1000);
        
        assert!(time_1 >= time_2);
        assert!(time_2 >= time_4);
    }

    #[test]
    fn test_speedup_calculation() {
        let speedup = ParallelProverCoordinator::calculate_speedup(4000, 1000);
        assert_eq!(speedup, 4.0);
    }

    #[test]
    fn test_speedup_calculation_partial() {
        let speedup = ParallelProverCoordinator::calculate_speedup(2000, 1000);
        assert_eq!(speedup, 2.0);
    }

    #[test]
    fn test_speedup_calculation_no_improvement() {
        let speedup = ParallelProverCoordinator::calculate_speedup(1000, 1000);
        assert_eq!(speedup, 1.0);
    }

    #[test]
    fn test_speedup_calculation_no_time() {
        let speedup = ParallelProverCoordinator::calculate_speedup(1000, 0);
        assert_eq!(speedup, 1.0);
    }
}

#[cfg(test)]
mod checkpoint_tests {
    use zk_fragment_prove::*;

    #[test]
    fn test_checkpoint_creation() {
        let checkpoint = ProvingCheckpoint::new("test_1".to_string(), 10);
        
        assert_eq!(checkpoint.completed_capsules, 0);
        assert_eq!(checkpoint.total_capsules, 10);
        assert_eq!(checkpoint.checkpoint_id, "test_1");
    }

    #[test]
    fn test_checkpoint_progress() {
        let mut checkpoint = ProvingCheckpoint::new("test".to_string(), 10);
        
        checkpoint.completed_capsules = 5;
        assert_eq!(checkpoint.progress_percentage(), 50.0);
        
        checkpoint.completed_capsules = 10;
        assert_eq!(checkpoint.progress_percentage(), 100.0);
    }

    #[test]
    fn test_checkpoint_progress_boundary() {
        let mut checkpoint = ProvingCheckpoint::new("test".to_string(), 10);
        
        checkpoint.completed_capsules = 0;
        assert_eq!(checkpoint.progress_percentage(), 0.0);
        
        checkpoint.completed_capsules = 1;
        assert_eq!(checkpoint.progress_percentage(), 10.0);
    }

    #[test]
    fn test_checkpoint_remaining_time() {
        let mut checkpoint = ProvingCheckpoint::new("test".to_string(), 10);
        
        checkpoint.completed_capsules = 2;
        checkpoint.elapsed_ms = 2000;
        
        let remaining = checkpoint.estimate_remaining_ms();
        
        // Rate: 2000ms / 2 = 1000ms per capsule
        // Remaining: 8 capsules * 1000ms = 8000ms
        assert_eq!(remaining, 8000);
    }

    #[test]
    fn test_checkpoint_remaining_time_zero_completed() {
        let checkpoint = ProvingCheckpoint::new("test".to_string(), 10);
        
        let remaining = checkpoint.estimate_remaining_ms();
        assert_eq!(remaining, 0);
    }

    #[test]
    fn test_checkpoint_high_rate() {
        let mut checkpoint = ProvingCheckpoint::new("test".to_string(), 100);
        
        checkpoint.completed_capsules = 50;
        checkpoint.elapsed_ms = 5000; // Very fast
        
        let remaining = checkpoint.estimate_remaining_ms();
        assert_eq!(remaining, 5000);
    }

    #[test]
    fn test_checkpoint_slow_rate() {
        let mut checkpoint = ProvingCheckpoint::new("test".to_string(), 100);
        
        checkpoint.completed_capsules = 10;
        checkpoint.elapsed_ms = 50000; // Slow
        
        let remaining = checkpoint.estimate_remaining_ms();
        assert_eq!(remaining, 450000); // 90 * 5000
    }
}

#[cfg(test)]
mod prover_integration_tests {
    use zk_fragment_prove::*;

    #[test]
    fn test_complete_proving_workflow() {
        // Fragment 0
        let metadata_0 = FragmentMetadata {
            fragment_id: 0,
            constraint_count: 100,
            input_boundary_count: 0,
            output_boundary_count: 1,
            execution_position: 0,
        };
        
        let capsule_0 = FragmentProofCapsule::new(metadata_0);
        let prover = FragmentProver::new(FragmentProverConfig::default());
        let proved_0 = prover.prove(capsule_0).unwrap();
        
        assert!(proved_0.is_proven);
        
        // Fragment 1
        let metadata_1 = FragmentMetadata {
            fragment_id: 1,
            constraint_count: 80,
            input_boundary_count: 1,
            output_boundary_count: 0,
            execution_position: 1,
        };
        
        let capsule_1 = FragmentProofCapsule::new(metadata_1);
        let proved_1 = prover.prove(capsule_1).unwrap();
        
        assert!(proved_1.is_proven);
        
        // Verify both are proven
        assert!(proved_0.get_proof().is_some());
        assert!(proved_1.get_proof().is_some());
    }

    #[test]
    fn test_batch_proving_consistency() {
        let config = FragmentProverConfig::default();
        let prover = FragmentProver::new(config);
        
        let capsules: Vec<_> = (0..3)
            .map(|i| {
                let metadata = FragmentMetadata {
                    fragment_id: i,
                    constraint_count: 100,
                    input_boundary_count: 0,
                    output_boundary_count: 0,
                    execution_position: i,
                };
                FragmentProofCapsule::new(metadata)
            })
            .collect();
        
        let proved = prover.prove_batch(capsules).unwrap();
        
        // All should be proven
        for capsule in &proved {
            assert!(capsule.is_proven);
        }
        
        // All should have unique fragment IDs
        let mut ids = Vec::new();
        for capsule in &proved {
            ids.push(capsule.metadata.fragment_id);
        }
        ids.sort();
        assert_eq!(ids, vec![0, 1, 2]);
    }

    #[test]
    fn test_prover_with_checkpointing() {
        let prover = FragmentProver::new(FragmentProverConfig::default());
        let mut checkpoint = ProvingCheckpoint::new("batch_1".to_string(), 5);
        
        for i in 0..5 {
            let metadata = FragmentMetadata {
                fragment_id: i,
                constraint_count: 100,
                input_boundary_count: 0,
                output_boundary_count: 0,
                execution_position: i,
            };
            
            let capsule = FragmentProofCapsule::new(metadata);
            let _proved = prover.prove(capsule).unwrap();
            
            checkpoint.completed_capsules += 1;
        }
        
        assert_eq!(checkpoint.progress_percentage(), 100.0);
    }
}