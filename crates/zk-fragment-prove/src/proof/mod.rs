//! Fragment proof structures and verification

pub mod fragment_proof;
pub mod verification;

pub use fragment_proof::CompleteFragmentProof;
pub use verification::{FragmentProofVerifier, FragmentProofVerification};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_proof_workflow() {
        let proof = CompleteFragmentProof::new(
            0,
            vec![0; 512],
            crate::ExecutionHash::from_bytes([1u8; 32]),
        )
        .with_constraint_count(1000)
        .with_generation_time(500);
        
        let verification = FragmentProofVerifier::verify(&proof).unwrap();
        assert!(verification.is_valid);
    }
}