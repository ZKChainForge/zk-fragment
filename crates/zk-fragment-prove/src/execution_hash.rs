//! Execution hash chain for fragment ordering
//!
//! The execution hash chain prevents reordering of fragments and
//! binds the entire computation together.

use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use std::fmt;

/// Genesis hash for the first fragment
/// This is a well-known constant that starts the chain
pub const GENESIS_HASH: &str = "0x0000000000000000000000000000000000000000000000000000000000000000";

/// Execution hash linking fragments together
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionHash {
    /// 256-bit hash as bytes
    pub value: [u8; 32],
}

impl ExecutionHash {
    /// Create from raw bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        ExecutionHash { value: bytes }
    }
    
    /// Get as hex string
    pub fn to_hex(&self) -> String {
        format!("0x{}", hex::encode(self.value))
    }
    
    /// Genesis hash (first fragment's prev hash)
    pub fn genesis() -> Self {
        ExecutionHash {
            value: [0u8; 32],
        }
    }
    
    /// Check if this is the genesis hash
    pub fn is_genesis(&self) -> bool {
        self.value == [0u8; 32]
    }
}

impl fmt::Display for ExecutionHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Builder for execution hashes
pub struct ExecutionHashBuilder {
    fragment_id: u32,
    previous_hash: ExecutionHash,
    boundary_commitments: Vec<[u8; 32]>,
}

impl ExecutionHashBuilder {
    /// Create a new builder
    pub fn new(fragment_id: u32) -> Self {
        ExecutionHashBuilder {
            fragment_id,
            previous_hash: ExecutionHash::genesis(),
            boundary_commitments: Vec::new(),
        }
    }
    
    /// Set the previous fragment's execution hash
    pub fn with_previous_hash(mut self, hash: ExecutionHash) -> Self {
        self.previous_hash = hash;
        self
    }
    
    /// Add boundary commitments to include in hash
    pub fn with_commitment(mut self, commitment: [u8; 32]) -> Self {
        self.boundary_commitments.push(commitment);
        self
    }
    
    /// Add multiple boundary commitments
    pub fn with_commitments(mut self, commitments: Vec<[u8; 32]>) -> Self {
        self.boundary_commitments.extend(commitments);
        self
    }
    
    /// Build the execution hash
    ///
    /// Formula:
    /// exec_hash = SHA256(
    ///     fragment_id ||
    ///     previous_hash ||
    ///     commitment_1 ||
    ///     commitment_2 ||
    ///     ...
    /// )
    pub fn build(self) -> ExecutionHash {
        let mut hasher = Sha256::new();
        
        // Hash fragment ID
        hasher.update(self.fragment_id.to_be_bytes());
        
        // Hash previous execution hash
        hasher.update(self.previous_hash.value);
        
        // Hash all boundary commitments in order
        for commitment in &self.boundary_commitments {
            hasher.update(commitment);
        }
        
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        
        ExecutionHash { value: hash }
    }
}

/// Chain link - represents one fragment in the hash chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainLink {
    /// Fragment identifier
    pub fragment_id: u32,
    
    /// Execution hash at this point
    pub execution_hash: ExecutionHash,
    
    /// Previous fragment's hash
    pub previous_hash: ExecutionHash,
}

impl ChainLink {
    pub fn new(
        fragment_id: u32,
        execution_hash: ExecutionHash,
        previous_hash: ExecutionHash,
    ) -> Self {
        ChainLink {
            fragment_id,
            execution_hash,
            previous_hash,
        }
    }
}

/// Verifier for execution hash chain
pub struct ChainVerifier {
    links: Vec<ChainLink>,
}

impl ChainVerifier {
    pub fn new() -> Self {
        ChainVerifier { links: Vec::new() }
    }
    
    /// Add a link to the chain
    pub fn add_link(&mut self, link: ChainLink) {
        self.links.push(link);
    }
    
    /// Verify chain integrity
    ///
    /// Checks:
    /// 1. First fragment has genesis hash as previous
    /// 2. Each subsequent fragment's prev_hash == previous fragment's exec_hash
    /// 3. Fragment IDs are in order
    pub fn verify(&self) -> bool {
        if self.links.is_empty() {
            return true; // Empty chain is valid
        }
        
        // Check first fragment
        if !self.links[0].previous_hash.is_genesis() {
            return false;
        }
        
        // Check subsequent fragments
        for i in 1..self.links.len() {
            let prev_exec_hash = self.links[i - 1].execution_hash;
            let curr_prev_hash = self.links[i].previous_hash;
            
            if prev_exec_hash != curr_prev_hash {
                return false;
            }
        }
        
        true
    }
    
    /// Get the final execution hash
    pub fn final_hash(&self) -> Option<ExecutionHash> {
        self.links.last().map(|link| link.execution_hash)
    }
}

impl Default for ChainVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_hash_genesis() {
        let hash = ExecutionHash::genesis();
        assert!(hash.is_genesis());
    }

    #[test]
    fn test_execution_hash_builder_single() {
        let hash = ExecutionHashBuilder::new(1).build();
        assert_ne!(hash.value, [0u8; 32]);
    }

    #[test]
    fn test_execution_hash_deterministic() {
        let h1 = ExecutionHashBuilder::new(1).build();
        let h2 = ExecutionHashBuilder::new(1).build();
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_execution_hash_different_fragments() {
        let h1 = ExecutionHashBuilder::new(1).build();
        let h2 = ExecutionHashBuilder::new(2).build();
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_chain_verifier_single_fragment() {
        let hash = ExecutionHashBuilder::new(1).build();
        let link = ChainLink::new(1, hash, ExecutionHash::genesis());
        
        let mut verifier = ChainVerifier::new();
        verifier.add_link(link);
        
        assert!(verifier.verify());
    }

    #[test]
    fn test_chain_verifier_two_fragments() {
        let h1 = ExecutionHashBuilder::new(1).build();
        let link1 = ChainLink::new(1, h1, ExecutionHash::genesis());
        
        let h2 = ExecutionHashBuilder::new(2)
            .with_previous_hash(h1)
            .build();
        let link2 = ChainLink::new(2, h2, h1);
        
        let mut verifier = ChainVerifier::new();
        verifier.add_link(link1);
        verifier.add_link(link2);
        
        assert!(verifier.verify());
    }

    #[test]
    fn test_chain_verifier_broken_chain() {
        let h1 = ExecutionHashBuilder::new(1).build();
        let link1 = ChainLink::new(1, h1, ExecutionHash::genesis());
        
        // Create link2 with wrong previous hash
        let h2 = ExecutionHashBuilder::new(2).build();
        let link2 = ChainLink::new(2, h2, ExecutionHash::genesis()); // Wrong!
        
        let mut verifier = ChainVerifier::new();
        verifier.add_link(link1);
        verifier.add_link(link2);
        
        assert!(!verifier.verify());
    }

    #[test]
    fn test_chain_final_hash() {
        let h1 = ExecutionHashBuilder::new(1).build();
        let link = ChainLink::new(1, h1, ExecutionHash::genesis());
        
        let mut verifier = ChainVerifier::new();
        verifier.add_link(link);
        
        assert_eq!(verifier.final_hash(), Some(h1));
    }
}