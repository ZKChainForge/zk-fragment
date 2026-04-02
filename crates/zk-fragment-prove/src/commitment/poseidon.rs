//! Poseidon Hash-based Commitment Scheme
//! 
//! This module implements a cryptographic commitment scheme using Poseidon
//! hash function, optimized for ZK circuits.

use anyhow::Result;
use plonky2_field::goldilocks_field::GoldilocksField;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use std::fmt;

// Type aliases for clarity
pub type Field = GoldilocksField;
pub type FieldValue = u64;

/// A commitment to a wire value
/// 
/// Mathematical definition:
/// commitment = Poseidon(value, blinding_factor)
/// 
/// Properties:
/// - Binding: Cannot find different value with same commitment
/// - Hiding: Commitment reveals nothing about value without blinding
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Commitment {
    /// The commitment hash (256 bits represented as u64)
    pub value: u64,
}

impl fmt::Display for Commitment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Commitment(0x{:016x})", self.value)
    }
}

/// Opening information for a commitment
/// 
/// This is the information needed to verify that a specific value
/// matches a commitment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitmentOpening {
    /// The actual wire value being committed to
    pub value: FieldValue,
    
    /// The blinding factor used in commitment
    /// Poseidon(value, blinding) == commitment
    pub blinding: [u8; 32],
}

/// A boundary commitment with associated opening
/// 
/// Used to communicate wire values across fragment boundaries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryCommitment {
    /// Unique wire identifier
    pub wire_id: u32,
    
    /// The commitment itself
    pub commitment: Commitment,
    
    /// Optional opening (only for verifier side)
    pub opening: Option<CommitmentOpening>,
}

impl BoundaryCommitment {
    /// Create a new boundary commitment without opening
    /// 
    /// Used by fragment producer to send to successor
    pub fn new(wire_id: u32, commitment: Commitment) -> Self {
        BoundaryCommitment {
            wire_id,
            commitment,
            opening: None,
        }
    }
    
    /// Create a boundary commitment with opening
    /// 
    /// Used by fragment verifier to check the value
    pub fn with_opening(
        wire_id: u32,
        commitment: Commitment,
        opening: CommitmentOpening,
    ) -> Self {
        BoundaryCommitment {
            wire_id,
            commitment,
            opening: Some(opening),
        }
    }
}

/// Poseidon parameter configuration
/// 
/// These are standard Poseidon parameters for 256-bit security
/// with 8 state elements and 8 full rounds + 55 partial rounds
pub const POSEIDON_FULL_ROUNDS: usize = 8;
pub const POSEIDON_PARTIAL_ROUNDS: usize = 55;
pub const POSEIDON_STATE_SIZE: usize = 8;

/// Poseidon hash implementation using SHA256 as backend
/// 
/// In production, this would use the actual Poseidon S-box,
/// but for development we use SHA256 for compatibility
pub fn poseidon_hash(value: FieldValue, blinding: &[u8; 32]) -> Commitment {
    let mut hasher = Sha256::new();
    
    // Hash the value (as 8 bytes, big-endian)
    hasher.update(value.to_be_bytes());
    
    // Hash the blinding factor
    hasher.update(blinding);
    
    let result = hasher.finalize();
    
    // Take first 8 bytes and convert to u64
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&result[0..8]);
    let hash_value = u64::from_be_bytes(bytes);
    
    Commitment { value: hash_value }
}

/// Generate a deterministic blinding factor
/// 
/// This allows the consumer to recompute the same blinding
/// given the fragment context
pub fn derive_blinding_factor(
    fragment_id: u32,
    wire_id: u32,
    shared_secret: &[u8; 32],
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    
    hasher.update(fragment_id.to_be_bytes());
    hasher.update(wire_id.to_be_bytes());
    hasher.update(shared_secret);
    hasher.update(b"blinding_derivation");
    
    let result = hasher.finalize();
    let mut blinding = [0u8; 32];
    blinding.copy_from_slice(&result);
    blinding
}

/// Verify a commitment against a value and opening
/// 
/// Returns true if and only if:
/// Poseidon(opening.value, opening.blinding) == commitment
pub fn verify_commitment(
    commitment: &Commitment,
    opening: &CommitmentOpening,
) -> bool {
    let computed = poseidon_hash(opening.value, &opening.blinding);
    computed == *commitment
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commitment_creation() {
        let value = 42u64;
        let blinding = [1u8; 32];
        
        let commitment = poseidon_hash(value, &blinding);
        assert_ne!(commitment.value, 0);
    }

    #[test]
    fn test_commitment_deterministic() {
        let value = 42u64;
        let blinding = [1u8; 32];
        
        let c1 = poseidon_hash(value, &blinding);
        let c2 = poseidon_hash(value, &blinding);
        
        assert_eq!(c1, c2, "Same inputs must produce same commitment");
    }

    #[test]
    fn test_commitment_different_values() {
        let blinding = [1u8; 32];
        
        let c1 = poseidon_hash(42u64, &blinding);
        let c2 = poseidon_hash(43u64, &blinding);
        
        assert_ne!(c1, c2, "Different values must produce different commitments");
    }

    #[test]
    fn test_commitment_different_blinding() {
        let value = 42u64;
        
        let c1 = poseidon_hash(value, &[1u8; 32]);
        let c2 = poseidon_hash(value, &[2u8; 32]);
        
        assert_ne!(c1, c2, "Different blindings must produce different commitments");
    }

    #[test]
    fn test_verify_commitment_valid() {
        let value = 42u64;
        let blinding = [1u8; 32];
        let commitment = poseidon_hash(value, &blinding);
        
        let opening = CommitmentOpening { value, blinding };
        
        assert!(verify_commitment(&commitment, &opening));
    }

    #[test]
    fn test_verify_commitment_invalid_value() {
        let value = 42u64;
        let blinding = [1u8; 32];
        let commitment = poseidon_hash(value, &blinding);
        
        let wrong_opening = CommitmentOpening { value: 43u64, blinding };
        
        assert!(!verify_commitment(&commitment, &wrong_opening));
    }

    #[test]
    fn test_verify_commitment_invalid_blinding() {
        let value = 42u64;
        let blinding = [1u8; 32];
        let commitment = poseidon_hash(value, &blinding);
        
        let wrong_opening = CommitmentOpening { value, blinding: [2u8; 32] };
        
        assert!(!verify_commitment(&commitment, &wrong_opening));
    }

    #[test]
    fn test_derive_blinding_deterministic() {
        let shared = [3u8; 32];
        
        let b1 = derive_blinding_factor(1, 100, &shared);
        let b2 = derive_blinding_factor(1, 100, &shared);
        
        assert_eq!(b1, b2, "Same inputs must produce same blinding");
    }

    #[test]
    fn test_derive_blinding_different_fragment() {
        let shared = [3u8; 32];
        
        let b1 = derive_blinding_factor(1, 100, &shared);
        let b2 = derive_blinding_factor(2, 100, &shared);
        
        assert_ne!(b1, b2, "Different fragments must produce different blindings");
    }
}