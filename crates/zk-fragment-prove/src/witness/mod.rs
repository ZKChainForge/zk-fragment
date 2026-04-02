//! Witness management for fragments
//!
//! Handles extraction, partitioning, and boundary witness preparation

pub mod extractor;
pub mod boundary_witness;

pub use extractor::{WitnessIndexMap, WitnessExtractor};
pub use boundary_witness::{BoundaryWireValue, BoundaryWitnessSegment, BoundaryWitnessBuilder};

#[cfg(test)]
mod tests {
    use super::*;

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
}