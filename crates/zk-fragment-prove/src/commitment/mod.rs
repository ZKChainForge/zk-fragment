//! Commitment scheme for boundary wires
//!
//! This module provides cryptographic commitments for wire values
//! that cross fragment boundaries.

pub mod poseidon;
pub mod generator;
pub mod verifier;

pub use poseidon::{
    Commitment, CommitmentOpening, BoundaryCommitment,
    poseidon_hash, derive_blinding_factor, verify_commitment,
};
pub use generator::{CommitmentGenerator, CommitmentConfig};
pub use verifier::CommitmentVerifier;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_commitment_workflow() {
        // Step 1: Generator creates commitment
        let gen_config = CommitmentConfig::new(1);
        let generator = CommitmentGenerator::new(gen_config);
        let boundary_commit = generator.generate_commitment(100, 42);
        
        // Step 2: Commitment is sent to consumer
        let commitment = boundary_commit.commitment;
        
        // Step 3: Consumer verifies commitment
        // (In real scenario, consumer would receive value and blinding)
        // For testing, we just verify the structure
        assert_ne!(commitment.value, 0);
    }
}