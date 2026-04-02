//! Prepare witness for boundary values
//! 
//! Handles boundary wire values and their openings

use crate::commitment::{CommitmentOpening, derive_blinding_factor};
use anyhow::Result;
use serde::{Serialize, Deserialize};

/// Wire value crossing a boundary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryWireValue {
    /// Wire identifier
    pub wire_id: u32,
    
    /// The actual value
    pub value: u64,
    
    /// Blinding factor for commitment
    pub blinding: [u8; 32],
}

impl BoundaryWireValue {
    pub fn new(wire_id: u32, value: u64, blinding: [u8; 32]) -> Self {
        BoundaryWireValue {
            wire_id,
            value,
            blinding,
        }
    }
    
    /// Convert to commitment opening
    pub fn to_opening(&self) -> CommitmentOpening {
        CommitmentOpening {
            value: self.value,
            blinding: self.blinding,
        }
    }
}

/// Witness segment specifically for boundaries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryWitnessSegment {
    /// Input boundary wires (received from previous fragment)
    pub input_boundaries: Vec<BoundaryWireValue>,
    
    /// Output boundary wires (to send to next fragment)
    pub output_boundaries: Vec<BoundaryWireValue>,
}

impl BoundaryWitnessSegment {
    pub fn new() -> Self {
        BoundaryWitnessSegment {
            input_boundaries: Vec::new(),
            output_boundaries: Vec::new(),
        }
    }
    
    pub fn with_input_boundary(mut self, boundary: BoundaryWireValue) -> Self {
        self.input_boundaries.push(boundary);
        self
    }
    
    pub fn with_input_boundaries(mut self, boundaries: Vec<BoundaryWireValue>) -> Self {
        self.input_boundaries.extend(boundaries);
        self
    }
    
    pub fn with_output_boundary(mut self, boundary: BoundaryWireValue) -> Self {
        self.output_boundaries.push(boundary);
        self
    }
    
    pub fn with_output_boundaries(mut self, boundaries: Vec<BoundaryWireValue>) -> Self {
        self.output_boundaries.extend(boundaries);
        self
    }
}

impl Default for BoundaryWitnessSegment {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for boundary witness
pub struct BoundaryWitnessBuilder {
    fragment_id: u32,
    shared_secret: [u8; 32],
}

impl BoundaryWitnessBuilder {
    pub fn new(fragment_id: u32, shared_secret: [u8; 32]) -> Self {
        BoundaryWitnessBuilder {
            fragment_id,
            shared_secret,
        }
    }
    
    /// Create input boundary from value
    pub fn create_input_boundary(
        &self,
        wire_id: u32,
        value: u64,
    ) -> BoundaryWireValue {
        let blinding = derive_blinding_factor(
            self.fragment_id,
            wire_id,
            &self.shared_secret,
        );
        
        BoundaryWireValue::new(wire_id, value, blinding)
    }
    
    /// Create multiple input boundaries
    pub fn create_input_boundaries(
        &self,
        wires: &[(u32, u64)],
    ) -> Vec<BoundaryWireValue> {
        wires
            .iter()
            .map(|(wire_id, value)| self.create_input_boundary(*wire_id, *value))
            .collect()
    }
    
    /// Create output boundary from value
    pub fn create_output_boundary(
        &self,
        wire_id: u32,
        value: u64,
    ) -> BoundaryWireValue {
        let blinding = derive_blinding_factor(
            self.fragment_id,
            wire_id,
            &self.shared_secret,
        );
        
        BoundaryWireValue::new(wire_id, value, blinding)
    }
    
    /// Create multiple output boundaries
    pub fn create_output_boundaries(
        &self,
        wires: &[(u32, u64)],
    ) -> Vec<BoundaryWireValue> {
        wires
            .iter()
            .map(|(wire_id, value)| self.create_output_boundary(*wire_id, *value))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boundary_wire_value() {
        let bwv = BoundaryWireValue::new(100, 42, [1u8; 32]);
        
        assert_eq!(bwv.wire_id, 100);
        assert_eq!(bwv.value, 42);
        assert_eq!(bwv.blinding, [1u8; 32]);
    }

    #[test]
    fn test_boundary_witness_segment() {
        let mut segment = BoundaryWitnessSegment::new();
        
        segment = segment.with_input_boundary(BoundaryWireValue::new(100, 42, [1u8; 32]));
        segment = segment.with_output_boundary(BoundaryWireValue::new(101, 43, [2u8; 32]));
        
        assert_eq!(segment.input_boundaries.len(), 1);
        assert_eq!(segment.output_boundaries.len(), 1);
    }

    #[test]
    fn test_boundary_witness_builder() {
        let builder = BoundaryWitnessBuilder::new(1, [3u8; 32]);
        
        let input = builder.create_input_boundary(100, 42);
        assert_eq!(input.wire_id, 100);
        assert_eq!(input.value, 42);
        
        let output = builder.create_output_boundary(101, 43);
        assert_eq!(output.wire_id, 101);
        assert_eq!(output.value, 43);
    }

    #[test]
    fn test_boundary_witness_builder_batch() {
        let builder = BoundaryWitnessBuilder::new(1, [3u8; 32]);
        
        let inputs = builder.create_input_boundaries(&[(100, 42), (101, 43)]);
        assert_eq!(inputs.len(), 2);
        assert_eq!(inputs[0].wire_id, 100);
        assert_eq!(inputs[1].wire_id, 101);
    }

    #[test]
    fn test_boundary_to_opening() {
        let bwv = BoundaryWireValue::new(100, 42, [1u8; 32]);
        let opening = bwv.to_opening();
        
        assert_eq!(opening.value, 42);
        assert_eq!(opening.blinding, [1u8; 32]);
    }
}