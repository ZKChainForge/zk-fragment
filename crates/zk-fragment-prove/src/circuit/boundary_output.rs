//! Output boundary generation gadget
//!
//! Generates commitments for output boundaries

use crate::commitment::{Commitment, poseidon_hash, derive_blinding_factor};
use serde::{Serialize, Deserialize};
use anyhow::Result;

/// Configuration for boundary output generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryOutputConfig {
    /// Fragment ID (for deterministic blinding)
    pub fragment_id: u32,
    
    /// Shared secret (for deterministic blinding)
    pub shared_secret: [u8; 32],
    
    /// Number of output boundaries
    pub num_outputs: usize,
}

impl BoundaryOutputConfig {
    pub fn new(fragment_id: u32, shared_secret: [u8; 32], num_outputs: usize) -> Self {
        BoundaryOutputConfig {
            fragment_id,
            shared_secret,
            num_outputs,
        }
    }
}

/// Gadget for generating output boundaries
pub struct BoundaryOutputGadget {
    config: BoundaryOutputConfig,
}

impl BoundaryOutputGadget {
    pub fn new(config: BoundaryOutputConfig) -> Self {
        BoundaryOutputGadget { config }
    }
    
    /// Generate commitments for output values
    ///
    /// In real circuit:
    /// 1. For each output wire value
    /// 2. Derive blinding factor
    /// 3. Compute commitment = Poseidon(value, blinding)
    /// 4. Output commitment
    pub fn generate_commitments(
        &self,
        wire_ids: &[u32],
        values: &[u64],
    ) -> Result<GenerationResult> {
        // Verify counts
        if wire_ids.len() != values.len() {
            anyhow::bail!("Wire ID/value count mismatch");
        }
        
        if wire_ids.len() != self.config.num_outputs {
            anyhow::bail!(
                "Output count mismatch: expected {}, got {}",
                self.config.num_outputs,
                wire_ids.len()
            );
        }
        
        let mut commitments = Vec::new();
        let mut constraint_count = 0u32;
        
        // Generate each commitment
        for (wire_id, value) in wire_ids.iter().zip(values.iter()) {
            // Derive blinding deterministically
            let blinding = derive_blinding_factor(
                self.config.fragment_id,
                *wire_id,
                &self.config.shared_secret,
            );
            
            // Compute commitment
            let commitment = poseidon_hash(*value, &blinding);
            
            commitments.push(OutputBoundary {
                wire_id: *wire_id,
                value: *value,
                commitment,
                blinding,
            });
            
            constraint_count += Self::CONSTRAINTS_PER_BOUNDARY;
        }
        
        Ok(GenerationResult {
            commitments,
            constraint_count,
        })
    }
    
    /// Generate single commitment
    pub fn generate_single(
        &self,
        wire_id: u32,
        value: u64,
    ) -> Result<OutputBoundary> {
        let blinding = derive_blinding_factor(
            self.config.fragment_id,
            wire_id,
            &self.config.shared_secret,
        );
        
        let commitment = poseidon_hash(value, &blinding);
        
        Ok(OutputBoundary {
            wire_id,
            value,
            commitment,
            blinding,
        })
    }
    
    /// Calculate constraint count
    pub fn constraint_count(&self) -> u32 {
        (self.config.num_outputs as u32) * Self::CONSTRAINTS_PER_BOUNDARY
    }
    
    // Constants
    const CONSTRAINTS_PER_BOUNDARY: u32 = 500; // Poseidon hash constraints
}

/// Output boundary with all information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputBoundary {
    pub wire_id: u32,
    pub value: u64,
    pub commitment: Commitment,
    pub blinding: [u8; 32],
}

/// Result of boundary generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResult {
    pub commitments: Vec<OutputBoundary>,
    pub constraint_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_boundary_output_different_values() {
        let config = BoundaryOutputConfig::new(1, [1u8; 32], 2);
        let gadget = BoundaryOutputGadget::new(config);
        
        let output1 = gadget.generate_single(100, 42).unwrap();
        let output2 = gadget.generate_single(100, 43).unwrap();
        
        assert_ne!(output1.commitment, output2.commitment);
    }

    #[test]
    fn test_boundary_output_count_mismatch() {
        let config = BoundaryOutputConfig::new(1, [1u8; 32], 2);
        let gadget = BoundaryOutputGadget::new(config);
        
        let result = gadget.generate_commitments(&[100], &[42]);
        assert!(result.is_err());
    }

    #[test]
    fn test_boundary_output_constraint_count() {
        let config = BoundaryOutputConfig::new(1, [1u8; 32], 4);
        let gadget = BoundaryOutputGadget::new(config);
        
        let count = gadget.constraint_count();
        assert_eq!(count, 4 * 500);
    }
}