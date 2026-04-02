//! Verify boundary commitments

use super::poseidon::{Commitment, CommitmentOpening, verify_commitment};
use anyhow::{Result, bail};
use std::collections::HashMap;

/// Verifier for boundary commitments
pub struct CommitmentVerifier {
    /// Map of wire_id -> expected commitment
    expected_commitments: HashMap<u32, Commitment>,
}

impl CommitmentVerifier {
    /// Create a new verifier
    pub fn new() -> Self {
        CommitmentVerifier {
            expected_commitments: HashMap::new(),
        }
    }
    
    /// Register an expected commitment
    pub fn register_commitment(&mut self, wire_id: u32, commitment: Commitment) {
        self.expected_commitments.insert(wire_id, commitment);
    }
    
    /// Register multiple commitments at once
    pub fn register_commitments(&mut self, commitments: &[(u32, Commitment)]) {
        for (wire_id, commitment) in commitments {
            self.register_commitment(*wire_id, *commitment);
        }
    }
    
    /// Verify a single opening against registered commitment
    ///
    /// Returns Ok(()) if commitment is valid, Err otherwise
    pub fn verify_opening(&self, wire_id: u32, opening: &CommitmentOpening) -> Result<()> {
        let expected = self
            .expected_commitments
            .get(&wire_id)
            .ok_or_else(|| anyhow::anyhow!("No expected commitment for wire {}", wire_id))?;
        
        if !verify_commitment(expected, opening) {
            bail!("Commitment verification failed for wire {}", wire_id);
        }
        
        Ok(())
    }
    
    /// Verify multiple openings
    pub fn verify_openings(&self, openings: &[(u32, CommitmentOpening)]) -> Result<()> {
        for (wire_id, opening) in openings {
            self.verify_opening(*wire_id, opening)?;
        }
        Ok(())
    }
    
    /// Get all registered commitments
    pub fn commitments(&self) -> &HashMap<u32, Commitment> {
        &self.expected_commitments
    }
    
    /// Clear all commitments
    pub fn clear(&mut self) {
        self.expected_commitments.clear();
    }
}

impl Default for CommitmentVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::poseidon::*;
    use super::*;

    #[test]
    fn test_verifier_verify_valid_opening() {
        let value = 42u64;
        let blinding = [1u8; 32];
        let commitment = poseidon_hash(value, &blinding);
        
        let mut verifier = CommitmentVerifier::new();
        verifier.register_commitment(100, commitment);
        
        let opening = CommitmentOpening { value, blinding };
        
        assert!(verifier.verify_opening(100, &opening).is_ok());
    }

    #[test]
    fn test_verifier_reject_invalid_opening() {
        let value = 42u64;
        let blinding = [1u8; 32];
        let commitment = poseidon_hash(value, &blinding);
        
        let mut verifier = CommitmentVerifier::new();
        verifier.register_commitment(100, commitment);
        
        let wrong_opening = CommitmentOpening { value: 43u64, blinding };
        
        assert!(verifier.verify_opening(100, &wrong_opening).is_err());
    }

    #[test]
    fn test_verifier_reject_unregistered_wire() {
        let verifier = CommitmentVerifier::new();
        let opening = CommitmentOpening { value: 42, blinding: [1u8; 32] };
        
        assert!(verifier.verify_opening(100, &opening).is_err());
    }

    #[test]
    fn test_verifier_batch_openings() {
        let blinding = [1u8; 32];
        let c1 = poseidon_hash(42u64, &blinding);
        let c2 = poseidon_hash(43u64, &blinding);
        
        let mut verifier = CommitmentVerifier::new();
        verifier.register_commitment(100, c1);
        verifier.register_commitment(101, c2);
        
        let openings = vec![
            (100, CommitmentOpening { value: 42, blinding }),
            (101, CommitmentOpening { value: 43, blinding }),
        ];
        
        assert!(verifier.verify_openings(&openings).is_ok());
    }
}