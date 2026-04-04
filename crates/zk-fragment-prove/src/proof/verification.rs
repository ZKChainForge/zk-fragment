//! Fragment proof verification

use super::fragment_proof::CompleteFragmentProof;
use anyhow::{Result, bail};
use serde::{Serialize, Deserialize};

/// Result of fragment proof verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentProofVerification {
    pub is_valid: bool,
    pub fragment_id: u32,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub verification_time_ms: u64,
}

impl FragmentProofVerification {
    pub fn valid(fragment_id: u32) -> Self {
        FragmentProofVerification {
            is_valid: true,
            fragment_id,
            errors: Vec::new(),
            warnings: Vec::new(),
            verification_time_ms: 0,
        }
    }
    
    pub fn with_error(mut self, error: String) -> Self {
        self.is_valid = false;
        self.errors.push(error);
        self
    }
    
    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }
    
    pub fn with_time(mut self, time_ms: u64) -> Self {
        self.verification_time_ms = time_ms;
        self
    }
}

/// Verifier for fragment proofs
pub struct FragmentProofVerifier;

impl FragmentProofVerifier {
    /// Verify a fragment proof
    pub fn verify(proof: &CompleteFragmentProof) -> Result<FragmentProofVerification> {
        let mut verification = FragmentProofVerification::valid(proof.fragment_id);
        
        // Check proof bytes exist
        if proof.proof_bytes.is_empty() {
            verification = verification.with_error("Proof bytes missing".to_string());
        }
        
        // Check execution hash
        if proof.execution_hash.value == [0u8; 32] {
            verification = verification.with_error("Invalid execution hash".to_string());
        }
        
        // Check constraints
        if proof.constraint_count == 0 {
            verification = verification.with_warning("Zero constraint count".to_string());
        }
        
        // Check commitment counts match
        if proof.input_commitments.is_empty() && proof.fragment_id > 0 {
            verification = verification.with_warning("No input commitments for non-first fragment".to_string());
        }
        
        Ok(verification)
    }
    
    /// Verify proof size is reasonable
    pub fn verify_proof_size(proof: &CompleteFragmentProof, max_size: usize) -> Result<()> {
        if proof.proof_size > max_size {
            bail!(
                "Proof size {} exceeds maximum {}",
                proof.proof_size,
                max_size
            );
        }
        Ok(())
    }
    
    /// Verify generation time is reasonable
    pub fn verify_generation_time(proof: &CompleteFragmentProof, max_time: u64) -> Result<()> {
        if proof.generation_time_ms > max_time {
            bail!(
                "Generation time {}ms exceeds maximum {}ms",
                proof.generation_time_ms,
                max_time
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fragment_proof_verification_valid() {
        let proof = CompleteFragmentProof::new(
            1,
            vec![0; 512],
            crate::ExecutionHash::from_bytes([1u8; 32]),
        ).with_constraint_count(1000);
        
        let result = FragmentProofVerifier::verify(&proof).unwrap();
        assert!(result.is_valid);
    }

    #[test]
    fn test_fragment_proof_verification_missing_bytes() {
        let proof = CompleteFragmentProof::new(
            1,
            vec![],
            crate::ExecutionHash::from_bytes([1u8; 32]),
        );
        
        let result = FragmentProofVerifier::verify(&proof).unwrap();
        assert!(!result.is_valid);
        assert!(result.errors.len() > 0);
    }

    #[test]
    fn test_fragment_proof_verification_size_check() {
        let proof = CompleteFragmentProof::new(
            1,
            vec![0; 1000],
            crate::ExecutionHash::from_bytes([1u8; 32]),
        );
        
        let result = FragmentProofVerifier::verify_proof_size(&proof, 500);
        assert!(result.is_err());
    }

    #[test]
    fn test_fragment_proof_verification_time_check() {
        let proof = CompleteFragmentProof::new(
            1,
            vec![0; 512],
            crate::ExecutionHash::from_bytes([1u8; 32]),
        ).with_generation_time(5000);
        
        let result = FragmentProofVerifier::verify_generation_time(&proof, 1000);
        assert!(result.is_err());
    }
}