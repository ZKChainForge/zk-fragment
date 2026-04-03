//! Merkle tree membership proof gadget (optional)
//!
//! For proving a fragment is part of a circuit fragment set

use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use anyhow::Result;

/// Configuration for Merkle membership
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleMembershipConfig {
    /// Fragment ID to prove membership of
    pub fragment_id: u32,
    
    /// Tree depth (number of siblings in proof)
    pub tree_depth: u32,
}

impl MerkleMembershipConfig {
    pub fn new(fragment_id: u32, tree_depth: u32) -> Self {
        MerkleMembershipConfig {
            fragment_id,
            tree_depth,
        }
    }
}

/// Merkle membership proof gadget
pub struct MerkleMembershipGadget {
    config: MerkleMembershipConfig,
}

impl MerkleMembershipGadget {
    pub fn new(config: MerkleMembershipConfig) -> Self {
        MerkleMembershipGadget { config }
    }
    
    /// Verify Merkle membership
    pub fn verify(
        &self,
        leaf_hash: [u8; 32],
        siblings: &[[u8; 32]],
        root: [u8; 32],
    ) -> Result<VerificationResult> {
        if siblings.len() != self.config.tree_depth as usize {
            anyhow::bail!(
                "Sibling count mismatch: expected {}, got {}",
                self.config.tree_depth,
                siblings.len()
            );
        }
        
        let mut current = leaf_hash;
        
        // Walk up the tree
        for sibling in siblings {
            current = Self::hash_pair(&current, sibling);
        }
        
        let is_valid = current == root;
        
        Ok(VerificationResult {
            is_valid,
            final_hash: current,
            constraint_count: (self.config.tree_depth as u32) * Self::CONSTRAINTS_PER_LEVEL,
        })
    }
    
    /// Hash a pair of nodes
    fn hash_pair(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(left);
        hasher.update(right);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
    
    /// Calculate constraint count
    pub fn constraint_count(&self) -> u32 {
        (self.config.tree_depth as u32) * Self::CONSTRAINTS_PER_LEVEL
    }
    
    const CONSTRAINTS_PER_LEVEL: u32 = 300;
}

/// Verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub is_valid: bool,
    pub final_hash: [u8; 32],
    pub constraint_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_membership_simple() {
        let config = MerkleMembershipConfig::new(0, 2);
        let gadget = MerkleMembershipGadget::new(config);
        
        let leaf = [1u8; 32];
        let sibling1 = [2u8; 32];
        let sibling2 = [3u8; 32];
        
        // Manually compute root
        let level1 = MerkleMembershipGadget::hash_pair(&leaf, &sibling1);
        let root = MerkleMembershipGadget::hash_pair(&level1, &sibling2);
        
        let result = gadget.verify(leaf, &[sibling1, sibling2], root).unwrap();
        assert!(result.is_valid);
    }

    #[test]
    fn test_merkle_membership_invalid() {
        let config = MerkleMembershipConfig::new(0, 2);
        let gadget = MerkleMembershipGadget::new(config);
        
        let leaf = [1u8; 32];
        let sibling1 = [2u8; 32];
        let sibling2 = [3u8; 32];
        
        let wrong_root = [99u8; 32];
        
        let result = gadget.verify(leaf, &[sibling1, sibling2], wrong_root).unwrap();
        assert!(!result.is_valid);
    }

    #[test]
    fn test_merkle_membership_depth_mismatch() {
        let config = MerkleMembershipConfig::new(0, 2);
        let gadget = MerkleMembershipGadget::new(config);
        
        let leaf = [1u8; 32];
        let sibling = [2u8; 32];
        let root = [3u8; 32];
        
        let result = gadget.verify(leaf, &[sibling], root);
        assert!(result.is_err());
    }

    #[test]
    fn test_merkle_membership_constraint_count() {
        let config = MerkleMembershipConfig::new(0, 5);
        let gadget = MerkleMembershipGadget::new(config);
        
        let count = gadget.constraint_count();
        assert_eq!(count, 5 * 300);
    }
}