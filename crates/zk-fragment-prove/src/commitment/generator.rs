//! Generate commitments for boundary wires

use super::poseidon::{
    Commitment, CommitmentOpening, BoundaryCommitment,
    poseidon_hash, derive_blinding_factor,
};
use anyhow::Result;
use serde::{Serialize, Deserialize};

/// Configuration for commitment generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitmentConfig {
    /// Fragment ID for this commitment generator
    pub fragment_id: u32,
    
    /// Shared secret for deterministic blinding
    pub shared_secret: [u8; 32],
    
    /// Whether to include openings (for testing)
    pub include_openings: bool,
}

impl CommitmentConfig {
    pub fn new(fragment_id: u32) -> Self {
        CommitmentConfig {
            fragment_id,
            shared_secret: [0u8; 32], // In production, use actual secret
            include_openings: false,
        }
    }
    
    pub fn with_secret(fragment_id: u32, secret: [u8; 32]) -> Self {
        CommitmentConfig {
            fragment_id,
            shared_secret: secret,
            include_openings: false,
        }
    }
}

/// Generator for boundary commitments
pub struct CommitmentGenerator {
    config: CommitmentConfig,
}

impl CommitmentGenerator {
    /// Create a new commitment generator
    pub fn new(config: CommitmentConfig) -> Self {
        CommitmentGenerator { config }
    }
    
    /// Generate commitment for a boundary wire value
    ///
    /// Process:
    /// 1. Derive blinding factor from fragment context
    /// 2. Compute Poseidon(value, blinding)
    /// 3. Create BoundaryCommitment
    /// 4. Optionally include opening (for testing)
    pub fn generate_commitment(
        &self,
        wire_id: u32,
        value: u64,
    ) -> BoundaryCommitment {
        // Step 1: Derive blinding factor
        let blinding = derive_blinding_factor(
            self.config.fragment_id,
            wire_id,
            &self.config.shared_secret,
        );
        
        // Step 2: Compute commitment
        let commitment = poseidon_hash(value, &blinding);
        
        // Step 3 & 4: Create boundary commitment
        if self.config.include_openings {
            let opening = CommitmentOpening { value, blinding };
            BoundaryCommitment::with_opening(wire_id, commitment, opening)
        } else {
            BoundaryCommitment::new(wire_id, commitment)
        }
    }
    
    /// Generate commitments for multiple wire values
    pub fn generate_commitments(
        &self,
        wire_values: &[(u32, u64)], // (wire_id, value) pairs
    ) -> Vec<BoundaryCommitment> {
        wire_values
            .iter()
            .map(|(wire_id, value)| self.generate_commitment(*wire_id, *value))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_commitment() {
        let config = CommitmentConfig::new(1);
        let gen = CommitmentGenerator::new(config);
        
        let bc = gen.generate_commitment(100, 42);
        
        assert_eq!(bc.wire_id, 100);
        assert_ne!(bc.commitment.value, 0);
    }

    #[test]
    fn test_generate_commitment_deterministic() {
        let config = CommitmentConfig::new(1);
        let gen = CommitmentGenerator::new(config);
        
        let bc1 = gen.generate_commitment(100, 42);
        let bc2 = gen.generate_commitment(100, 42);
        
        assert_eq!(bc1.commitment, bc2.commitment);
    }

    #[test]
    fn test_generate_commitment_different_wires() {
        let config = CommitmentConfig::new(1);
        let gen = CommitmentGenerator::new(config);
        
        let bc1 = gen.generate_commitment(100, 42);
        let bc2 = gen.generate_commitment(101, 42);
        
        assert_ne!(bc1.commitment, bc2.commitment);
    }

    #[test]
    fn test_generate_commitment_with_opening() {
        let mut config = CommitmentConfig::new(1);
        config.include_openings = true;
        let gen = CommitmentGenerator::new(config);
        
        let bc = gen.generate_commitment(100, 42);
        
        assert!(bc.opening.is_some());
        let opening = bc.opening.unwrap();
        assert_eq!(opening.value, 42);
    }

    #[test]
    fn test_generate_commitments_batch() {
        let config = CommitmentConfig::new(1);
        let gen = CommitmentGenerator::new(config);
        
        let wire_values = vec![
            (100, 42u64),
            (101, 43u64),
            (102, 44u64),
        ];
        
        let bcs = gen.generate_commitments(&wire_values);
        
        assert_eq!(bcs.len(), 3);
        assert_eq!(bcs[0].wire_id, 100);
        assert_eq!(bcs[1].wire_id, 101);
        assert_eq!(bcs[2].wire_id, 102);
    }
}