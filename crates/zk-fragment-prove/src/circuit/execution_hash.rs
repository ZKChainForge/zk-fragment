//! Execution hash computation gadget
//!
//! Computes execution hash linking fragments together

use crate::execution_hash::ExecutionHash;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use anyhow::Result;

/// Configuration for execution hash computation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionHashConfig {
    /// Fragment ID
    pub fragment_id: u32,
    
    /// Number of input boundaries
    pub num_input_boundaries: usize,
    
    /// Number of output boundaries
    pub num_output_boundaries: usize,
}

impl ExecutionHashConfig {
    pub fn new(
        fragment_id: u32,
        num_input_boundaries: usize,
        num_output_boundaries: usize,
    ) -> Self {
        ExecutionHashConfig {
            fragment_id,
            num_input_boundaries,
            num_output_boundaries,
        }
    }
}

/// Gadget for computing execution hash
pub struct ExecutionHashGadget {
    config: ExecutionHashConfig,
}

impl ExecutionHashGadget {
    pub fn new(config: ExecutionHashConfig) -> Self {
        ExecutionHashGadget { config }
    }
    
    /// Compute execution hash
    ///
    /// In circuit:
    /// Hash = SHA256(
    ///     fragment_id ||
    ///     previous_hash ||
    ///     input_boundary_1 ||
    ///     input_boundary_2 ||
    ///     ...
    ///     output_boundary_1 ||
    ///     output_boundary_2 ||
    ///     ...
    /// )
    pub fn compute(
        &self,
        previous_hash: ExecutionHash,
        input_commitments: &[[u8; 32]],
        output_commitments: &[[u8; 32]],
    ) -> Result<ComputationResult> {
        // Verify counts
        if input_commitments.len() != self.config.num_input_boundaries {
            anyhow::bail!(
                "Input boundary count mismatch: expected {}, got {}",
                self.config.num_input_boundaries,
                input_commitments.len()
            );
        }
        
        if output_commitments.len() != self.config.num_output_boundaries {
            anyhow::bail!(
                "Output boundary count mismatch: expected {}, got {}",
                self.config.num_output_boundaries,
                output_commitments.len()
            );
        }
        
        // Compute hash
        let mut hasher = Sha256::new();
        
        // Hash fragment ID
        hasher.update(self.config.fragment_id.to_be_bytes());
        
        // Hash previous hash
        hasher.update(previous_hash.value);
        
        // Hash input commitments
        for commitment in input_commitments {
            hasher.update(commitment);
        }
        
        // Hash output commitments
        for commitment in output_commitments {
            hasher.update(commitment);
        }
        
        let result = hasher.finalize();
        let mut hash_value = [0u8; 32];
        hash_value.copy_from_slice(&result);
        
        Ok(ComputationResult {
            execution_hash: ExecutionHash::from_bytes(hash_value),
            constraint_count: Self::CONSTRAINT_COUNT,
        })
    }
    
    /// Calculate constraint count
    pub fn constraint_count(&self) -> u32 {
        Self::CONSTRAINT_COUNT
    }
    
    // Constants
    const CONSTRAINT_COUNT: u32 = 1000; // SHA256 constraints in circuit
}

/// Result of hash computation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputationResult {
    pub execution_hash: ExecutionHash,
    pub constraint_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_execution_hash_gadget_deterministic() {
        let config = ExecutionHashConfig::new(0, 0, 0);
        let gadget = ExecutionHashGadget::new(config);
        
        let result1 = gadget.compute(
            ExecutionHash::genesis(),
            &[],
            &[],
        ).unwrap();
        
        let result2 = gadget.compute(
            ExecutionHash::genesis(),
            &[],
            &[],
        ).unwrap();
        
        assert_eq!(result1.execution_hash, result2.execution_hash);
    }

    #[test]
    fn test_execution_hash_gadget_input_mismatch() {
        let config = ExecutionHashConfig::new(1, 2, 0);
        let gadget = ExecutionHashGadget::new(config);
        
        let result = gadget.compute(
            ExecutionHash::genesis(),
            &[[1u8; 32]], // Only 1, but expect 2
            &[],
        );
        
        assert!(result.is_err());
    }

    #[test]
    fn test_execution_hash_gadget_output_mismatch() {
        let config = ExecutionHashConfig::new(1, 0, 2);
        let gadget = ExecutionHashGadget::new(config);
        
        let result = gadget.compute(
            ExecutionHash::genesis(),
            &[],
            &[[1u8; 32]], // Only 1, but expect 2
        );
        
        assert!(result.is_err());
    }
}