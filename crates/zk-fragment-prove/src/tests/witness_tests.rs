//! Tests for witness management and partitioning

#[cfg(test)]
mod witness_index_map_tests {
    use zk_fragment_prove::witness::*;

    #[test]
    fn test_witness_index_map_creation() {
        let map = WitnessIndexMap::new();
        assert_eq!(map.fragment_witness_size, 0);
        assert_eq!(map.index_mapping.len(), 0);
    }

    #[test]
    fn test_witness_index_map_add_mapping() {
        let mut map = WitnessIndexMap::new();
        
        map.add_mapping(0, 0);
        assert_eq!(map.fragment_witness_size, 1);
        
        map.add_mapping(5, 1);
        assert_eq!(map.fragment_witness_size, 2);
        
        map.add_mapping(10, 2);
        assert_eq!(map.fragment_witness_size, 3);
    }

    #[test]
    fn test_witness_index_map_get_mapping() {
        let mut map = WitnessIndexMap::new();
        map.add_mapping(0, 0);
        map.add_mapping(2, 1);
        map.add_mapping(5, 2);
        
        assert_eq!(map.get_fragment_index(0), Some(0));
        assert_eq!(map.get_fragment_index(2), Some(1));
        assert_eq!(map.get_fragment_index(5), Some(2));
        assert_eq!(map.get_fragment_index(3), None);
    }

    #[test]
    fn test_witness_index_map_sparse_mapping() {
        let mut map = WitnessIndexMap::new();
        
        // Map with gaps
        for i in [0, 3, 7, 15].iter() {
            map.add_mapping(*i, *i / 3);
        }
        
        assert_eq!(map.fragment_witness_size, 6); // 15/3 = 5, so size = 6
        assert_eq!(map.index_mapping.len(), 4);
    }

    #[test]
    fn test_witness_index_map_dense_mapping() {
        let mut map = WitnessIndexMap::new();
        
        for i in 0..10 {
            map.add_mapping(i, i);
        }
        
        assert_eq!(map.fragment_witness_size, 10);
        assert_eq!(map.index_mapping.len(), 10);
    }

    #[test]
    fn test_witness_index_map_overwrite() {
        let mut map = WitnessIndexMap::new();
        
        map.add_mapping(0, 0);
        map.add_mapping(0, 1); // Overwrite
        
        assert_eq!(map.get_fragment_index(0), Some(1));
        assert_eq!(map.index_mapping.len(), 1);
    }

    #[test]
    fn test_witness_index_map_default() {
        let map = WitnessIndexMap::default();
        assert_eq!(map.fragment_witness_size, 0);
    }
}

#[cfg(test)]
mod witness_extractor_tests {
    use zk_fragment_prove::witness::*;

    #[test]
    fn test_witness_extractor_simple() {
        let mut map = WitnessIndexMap::new();
        map.add_mapping(0, 0);
        map.add_mapping(1, 1);
        map.add_mapping(2, 2);
        
        let full_witness = vec![10, 20, 30, 40, 50];
        let extractor = WitnessExtractor::new(map);
        
        let fragment_witness = extractor.extract(&full_witness).unwrap();
        
        assert_eq!(fragment_witness, vec![10, 20, 30]);
    }

    #[test]
    fn test_witness_extractor_sparse() {
        let mut map = WitnessIndexMap::new();
        map.add_mapping(0, 0);
        map.add_mapping(3, 1);
        map.add_mapping(7, 2);
        
        let full_witness = vec![10, 20, 30, 40, 50, 60, 70, 80];
        let extractor = WitnessExtractor::new(map);
        
        let fragment_witness = extractor.extract(&full_witness).unwrap();
        
        assert_eq!(fragment_witness.len(), 3);
        assert_eq!(fragment_witness[0], 10);
        assert_eq!(fragment_witness[1], 40);
        assert_eq!(fragment_witness[2], 80);
    }

    #[test]
    fn test_witness_extractor_single() {
        let mut map = WitnessIndexMap::new();
        map.add_mapping(5, 0);
        
        let full_witness = vec![10, 20, 30, 40, 50, 60];
        let extractor = WitnessExtractor::new(map);
        
        let fragment_witness = extractor.extract(&full_witness).unwrap();
        
        assert_eq!(fragment_witness.len(), 1);
        assert_eq!(fragment_witness[0], 60);
    }

    #[test]
    fn test_witness_extractor_large() {
        let mut map = WitnessIndexMap::new();
        
        // Extract every 3rd element
        for i in 0..100 {
            if i % 3 == 0 {
                map.add_mapping(i, i / 3);
            }
        }
        
        let full_witness: Vec<u64> = (0..100).collect();
        let extractor = WitnessExtractor::new(map);
        
        let fragment_witness = extractor.extract(&full_witness).unwrap();
        
        assert_eq!(fragment_witness.len(), 34); // ~100/3
    }

    #[test]
    fn test_witness_extractor_out_of_bounds() {
        let mut map = WitnessIndexMap::new();
        map.add_mapping(10, 0);
        
        let full_witness = vec![1, 2, 3];
        let extractor = WitnessExtractor::new(map);
        
        let result = extractor.extract(&full_witness);
        assert!(result.is_err());
    }

    #[test]
    fn test_witness_extractor_batch() {
        let map = WitnessIndexMap::new();
        let full_witness = vec![10, 20, 30, 40, 50];
        let extractor = WitnessExtractor::new(map);
        
        let batch = extractor.extract_batch(&full_witness, &[0, 2, 4]).unwrap();
        assert_eq!(batch, vec![10, 30, 50]);
    }

    #[test]
    fn test_witness_extractor_batch_out_of_bounds() {
        let map = WitnessIndexMap::new();
        let full_witness = vec![10, 20, 30];
        let extractor = WitnessExtractor::new(map);
        
        let result = extractor.extract_batch(&full_witness, &[0, 2, 5]);
        assert!(result.is_err());
    }

    #[test]
    fn test_witness_extractor_empty() {
        let map = WitnessIndexMap::new();
        let full_witness = vec![10, 20, 30];
        let extractor = WitnessExtractor::new(map);
        
        let fragment_witness = extractor.extract(&full_witness).unwrap();
        assert_eq!(fragment_witness.len(), 0);
    }
}

#[cfg(test)]
mod boundary_wire_value_tests {
    use zk_fragment_prove::witness::*;

    #[test]
    fn test_boundary_wire_value_creation() {
        let bwv = BoundaryWireValue::new(100, 42, [1u8; 32]);
        
        assert_eq!(bwv.wire_id, 100);
        assert_eq!(bwv.value, 42);
    }

    #[test]
    fn test_boundary_wire_value_to_opening() {
        let bwv = BoundaryWireValue::new(100, 42, [1u8; 32]);
        let opening = bwv.to_opening();
        
        assert_eq!(opening.value, 42);
        assert_eq!(opening.blinding, [1u8; 32]);
    }

    #[test]
    fn test_boundary_wire_value_multiple() {
        let mut values = Vec::new();
        for i in 0..5 {
            let bwv = BoundaryWireValue::new(100 + i, 40 + i as u64, [i as u8; 32]);
            values.push(bwv);
        }
        
        assert_eq!(values.len(), 5);
        assert_eq!(values[0].value, 40);
        assert_eq!(values[4].value, 44);
    }
}

#[cfg(test)]
mod boundary_witness_segment_tests {
    use zk_fragment_prove::witness::*;

    #[test]
    fn test_boundary_witness_segment_creation() {
        let segment = BoundaryWitnessSegment::new();
        
        assert_eq!(segment.input_boundaries.len(), 0);
        assert_eq!(segment.output_boundaries.len(), 0);
    }

    #[test]
    fn test_boundary_witness_segment_with_inputs() {
        let inputs = vec![
            BoundaryWireValue::new(100, 42, [1u8; 32]),
            BoundaryWireValue::new(101, 43, [2u8; 32]),
        ];
        
        let segment = BoundaryWitnessSegment::new()
            .with_input_boundaries(inputs);
        
        assert_eq!(segment.input_boundaries.len(), 2);
    }

    #[test]
    fn test_boundary_witness_segment_with_outputs() {
        let outputs = vec![
            BoundaryWireValue::new(200, 100, [3u8; 32]),
        ];
        
        let segment = BoundaryWitnessSegment::new()
            .with_output_boundaries(outputs);
        
        assert_eq!(segment.output_boundaries.len(), 1);
    }

    #[test]
    fn test_boundary_witness_segment_mixed() {
        let inputs = vec![
            BoundaryWireValue::new(100, 42, [1u8; 32]),
        ];
        let outputs = vec![
            BoundaryWireValue::new(200, 100, [3u8; 32]),
        ];
        
        let segment = BoundaryWitnessSegment::new()
            .with_input_boundaries(inputs)
            .with_output_boundaries(outputs);
        
        assert_eq!(segment.input_boundaries.len(), 1);
        assert_eq!(segment.output_boundaries.len(), 1);
    }

    #[test]
    fn test_boundary_witness_segment_single_boundary() {
        let boundary = BoundaryWireValue::new(100, 42, [1u8; 32]);
        
        let segment = BoundaryWitnessSegment::new()
            .with_input_boundary(boundary);
        
        assert_eq!(segment.input_boundaries.len(), 1);
    }

    #[test]
    fn test_boundary_witness_segment_default() {
        let segment = BoundaryWitnessSegment::default();
        assert_eq!(segment.input_boundaries.len(), 0);
    }
}

#[cfg(test)]
mod boundary_witness_builder_tests {
    use zk_fragment_prove::witness::*;

    #[test]
    fn test_boundary_witness_builder_creation() {
        let builder = BoundaryWitnessBuilder::new(1, [3u8; 32]);
        
        let input = builder.create_input_boundary(100, 42);
        assert_eq!(input.wire_id, 100);
        assert_eq!(input.value, 42);
    }

    #[test]
    fn test_boundary_witness_builder_output() {
        let builder = BoundaryWitnessBuilder::new(1, [3u8; 32]);
        
        let output = builder.create_output_boundary(200, 100);
        assert_eq!(output.wire_id, 200);
        assert_eq!(output.value, 100);
    }

    #[test]
    fn test_boundary_witness_builder_batch_inputs() {
        let builder = BoundaryWitnessBuilder::new(1, [3u8; 32]);
        
        let inputs = builder.create_input_boundaries(&[
            (100, 42),
            (101, 43),
            (102, 44),
        ]);
        
        assert_eq!(inputs.len(), 3);
        assert_eq!(inputs[0].value, 42);
        assert_eq!(inputs[2].value, 44);
    }

    #[test]
    fn test_boundary_witness_builder_batch_outputs() {
        let builder = BoundaryWitnessBuilder::new(1, [3u8; 32]);
        
        let outputs = builder.create_output_boundaries(&[
            (200, 100),
            (201, 101),
        ]);
        
        assert_eq!(outputs.len(), 2);
    }

    #[test]
    fn test_boundary_witness_builder_deterministic() {
        let builder = BoundaryWitnessBuilder::new(1, [3u8; 32]);
        
        let input1 = builder.create_input_boundary(100, 42);
        let input2 = builder.create_input_boundary(100, 42);
        
        assert_eq!(input1.blinding, input2.blinding);
    }

    #[test]
    fn test_boundary_witness_builder_different_fragments() {
        let builder1 = BoundaryWitnessBuilder::new(1, [3u8; 32]);
        let builder2 = BoundaryWitnessBuilder::new(2, [3u8; 32]);
        
        let input1 = builder1.create_input_boundary(100, 42);
        let input2 = builder2.create_input_boundary(100, 42);
        
        assert_ne!(input1.blinding, input2.blinding);
    }
}

#[cfg(test)]
mod fragment_witness_tests {
    use zk_fragment_prove::*;

    #[test]
    fn test_fragment_witness_creation() {
        let witness = FragmentWitness::new();
        
        assert_eq!(witness.local_witness.len(), 0);
        assert_eq!(witness.public_inputs.len(), 0);
        assert_eq!(witness.input_boundaries.len(), 0);
    }

    #[test]
    fn test_fragment_witness_with_local() {
        let witness = FragmentWitness::new()
            .with_local_witness(vec![1, 2, 3, 4, 5]);
        
        assert_eq!(witness.local_witness.len(), 5);
    }

    #[test]
    fn test_fragment_witness_with_public_inputs() {
        let witness = FragmentWitness::new()
            .with_public_inputs(vec![10, 20, 30]);
        
        assert_eq!(witness.public_inputs.len(), 3);
    }

    #[test]
    fn test_fragment_witness_with_boundaries() {
        let boundaries = vec![
            CommitmentOpening { value: 42, blinding: [1u8; 32] },
            CommitmentOpening { value: 43, blinding: [2u8; 32] },
        ];
        
        let witness = FragmentWitness::new()
            .with_input_boundaries(boundaries);
        
        assert_eq!(witness.input_boundaries.len(), 2);
    }

    #[test]
    fn test_fragment_witness_complete() {
        let witness = FragmentWitness::new()
            .with_local_witness(vec![1, 2, 3])
            .with_public_inputs(vec![10, 20])
            .with_input_boundaries(vec![
                CommitmentOpening { value: 42, blinding: [1u8; 32] }
            ]);
        
        assert_eq!(witness.local_witness.len(), 3);
        assert_eq!(witness.public_inputs.len(), 2);
        assert_eq!(witness.input_boundaries.len(), 1);
    }

    #[test]
    fn test_fragment_witness_default() {
        let witness = FragmentWitness::default();
        assert_eq!(witness.local_witness.len(), 0);
    }
}

#[cfg(test)]
mod witness_integration_tests {
    use zk_fragment_prove::*;

    #[test]
    fn test_complete_witness_workflow() {
        // Step 1: Create index map
        let mut index_map = WitnessIndexMap::new();
        index_map.add_mapping(0, 0);
        index_map.add_mapping(2, 1);
        index_map.add_mapping(5, 2);
        
        // Step 2: Extract from full witness
        let full_witness = vec![10, 20, 30, 40, 50, 60];
        let extractor = WitnessExtractor::new(index_map);
        let fragment_witness = extractor.extract(&full_witness).unwrap();
        
        assert_eq!(fragment_witness, vec![10, 30, 60]);
        
        // Step 3: Create boundary witness
        let builder = BoundaryWitnessBuilder::new(1, [1u8; 32]);
        let boundary_segment = BoundaryWitnessSegment::new()
            .with_input_boundaries(vec![
                builder.create_input_boundary(100, 42),
                builder.create_input_boundary(101, 43),
            ])
            .with_output_boundaries(vec![
                builder.create_output_boundary(102, 44),
            ]);
        
        assert_eq!(boundary_segment.input_boundaries.len(), 2);
        assert_eq!(boundary_segment.output_boundaries.len(), 1);
    }

    #[test]
    fn test_multi_fragment_witness_partitioning() {
        // Full witness: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
        let full_witness: Vec<u64> = (0..10).collect();
        
        // Fragment 0: needs indices 0, 2, 4
        let mut map_0 = WitnessIndexMap::new();
        map_0.add_mapping(0, 0);
        map_0.add_mapping(2, 1);
        map_0.add_mapping(4, 2);
        
        let extractor_0 = WitnessExtractor::new(map_0);
        let witness_0 = extractor_0.extract(&full_witness).unwrap();
        
        assert_eq!(witness_0, vec![0, 2, 4]);
        
        // Fragment 1: needs indices 1, 3, 5
        let mut map_1 = WitnessIndexMap::new();
        map_1.add_mapping(1, 0);
        map_1.add_mapping(3, 1);
        map_1.add_mapping(5, 2);
        
        let extractor_1 = WitnessExtractor::new(map_1);
        let witness_1 = extractor_1.extract(&full_witness).unwrap();
        
        assert_eq!(witness_1, vec![1, 3, 5]);
    }

    #[test]
    fn test_witness_with_boundaries_integration() {
        let mut index_map = WitnessIndexMap::new();
        for i in 0..5 {
            index_map.add_mapping(i, i);
        }
        
        let full_witness: Vec<u64> = (0..10).collect();
        let extractor = WitnessExtractor::new(index_map);
        let local_witness = extractor.extract(&full_witness).unwrap();
        
        let builder = BoundaryWitnessBuilder::new(1, [1u8; 32]);
        let boundaries = builder.create_input_boundaries(&[(100, 42)]);
        
        let fragment_witness = FragmentWitness::new()
            .with_local_witness(local_witness)
            .with_input_boundaries(boundaries)
            .with_public_inputs(vec![99]);
        
        assert_eq!(fragment_witness.local_witness.len(), 5);
        assert_eq!(fragment_witness.input_boundaries.len(), 1);
        assert_eq!(fragment_witness.public_inputs.len(), 1);
    }

    #[test]
    fn test_large_witness_partitioning() {
        let full_witness: Vec<u64> = (0..1000).collect();
        
        let mut map = WitnessIndexMap::new();
        for i in (0..1000).step_by(10) {
            map.add_mapping(i, i / 10);
        }
        
        let extractor = WitnessExtractor::new(map);
        let fragment_witness = extractor.extract(&full_witness).unwrap();
        
        assert_eq!(fragment_witness.len(), 100);
        assert_eq!(fragment_witness[0], 0);
        assert_eq!(fragment_witness[99], 990);
    }
}