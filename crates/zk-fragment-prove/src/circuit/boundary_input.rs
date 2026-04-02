//! Input boundary verification gadget
//!
//! Verifies that input boundary values match their commitments

use crate::commitment::{Commitment, CommitmentOpening, poseidon_hash};
use serde::{Serialize, Deserialize};
use anyhow::{Result, bail};

/// Configuration for boundary input verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryInputConfig {
    /// Number of input boundaries to verify
    pub num_inputs: usize,
    
    /// Maximum value for any boundary
    pub max_value: u64,
}

impl BoundaryInputConfig {
    pub fn new(num_inputs: usize) -> Self {
        BoundaryInputConfig {
            num_inputs,
            max_value: u64::MAX,
        }
    }
}

/// Gadget for verifying input boundaries
pub struct BoundaryInputGadget {
    config: BoundaryInputConfig,
}

impl BoundaryInputGadget {
    pub fn new(config: BoundaryInputConfig) -> Self {
        BoundaryInputGadget { config }
    }
    
    /// Verify all input boundaries
    ///
    /// In a real circuit, this would be constraints.
    /// In our implementation, we simulate it.
    pub fn verify_all(
        &self,
        commitments: &[Commitment],
        openings: &[CommitmentOpening],
    ) -> Result<VerificationResult> {
        // Check counts match
        if commitments.len() != openings.len() {
            bail!(
                "Commitment/opening count mismatch: {} vs {}",
                commitments.len(),
                openings.len()
            );
        }
        
        if commitments.len() != self.config.num_inputs {
            bail!(
                "Input count mismatch: expected {}, got {}",
                self.config.num_inputs,
                commitments.len()
            );
        }
        
        let mut verified_values = Vec::new();
        let mut constraint_count = 0u32;
        
        // Verify each boundary
        for (i, (commitment, opening)) in commitments.iter().zip(openings.iter()).enumerate() {
            // Check value is within range
            if opening.value > self.config.max_value {
                bail!(
                    "Input boundary {} value out of range: {}",
                    i,
                    opening.value
                );
            }
            
            // Verify commitment
            let computed = poseidon_hash(opening.value, &opening.blinding);
            if computed != *commitment {
                bail!(
                    "Input boundary {} commitment mismatch",
                    i
                );
            }
            
            verified_values.push(opening.value);
            constraint_count += Self::CONSTRAINTS_PER_BOUNDARY;
        }
        
        Ok(VerificationResult {
            is_valid: true,
            verified_values,
            constraint_count,
            errors: vec![],
        })
    }
    
    /// Verify single input boundary
    pub fn verify_single(
        &self,
        commitment: &Commitment,
        opening: &CommitmentOpening,
    ) -> Result<VerificationResult> {
        self.verify_all(&[*commitment], &[opening.clone()])
    }
    
    /// Calculate constraint count
    pub fn constraint_count(&self) -> u32 {
        (self.config.num_inputs as u32) * Self::CONSTRAINTS_PER_BOUNDARY
    }
    
    // Constants
    const CONSTRAINTS_PER_BOUNDARY: u32 = 500; // Poseidon hash constraints
}

/// Result of boundary input verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub is_valid: bool,
    pub verified_values: Vec<u64>,
    pub constraint_count: u32,
    pub errors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_boundary_input_gadget_constraint_count() {
        let config = BoundaryInputConfig::new(5);
        let gadget = BoundaryInputGadget::new(config);
        
        let count = gadget.constraint_count();
        assert_eq!(count, 5 * 500);
    }
}