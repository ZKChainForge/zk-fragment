//! Fragment proof structure and metadata

use serde::{Serialize, Deserialize};
use crate::execution_hash::ExecutionHash;

/// Complete fragment proof with all metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteFragmentProof {
    /// Fragment ID
    pub fragment_id: u32,
    
    /// The actual ZK proof bytes
    pub proof_bytes: Vec<u8>,
    
    /// Proof size in bytes
    pub proof_size: usize,
    
    /// Execution hash from this fragment
    pub execution_hash: ExecutionHash,
    
    /// Input boundary commitments (as public inputs)
    pub input_commitments: Vec<[u8; 32]>,
    
    /// Output boundary commitments (as public outputs)
    pub output_commitments: Vec<[u8; 32]>,
    
    /// Time to generate proof (milliseconds)
    pub generation_time_ms: u64,
    
    /// Constraints in this proof
    pub constraint_count: u32,
}

impl CompleteFragmentProof {
    pub fn new(
        fragment_id: u32,
        proof_bytes: Vec<u8>,
        execution_hash: ExecutionHash,
    ) -> Self {
        let proof_size = proof_bytes.len();
        
        CompleteFragmentProof {
            fragment_id,
            proof_bytes,
            proof_size,
            execution_hash,
            input_commitments: Vec::new(),
            output_commitments: Vec::new(),
            generation_time_ms: 0,
            constraint_count: 0,
        }
    }
    
    pub fn with_input_commitments(mut self, commitments: Vec<[u8; 32]>) -> Self {
        self.input_commitments = commitments;
        self
    }
    
    pub fn with_output_commitments(mut self, commitments: Vec<[u8; 32]>) -> Self {
        self.output_commitments = commitments;
        self
    }
    
    pub fn with_generation_time(mut self, time_ms: u64) -> Self {
        self.generation_time_ms = time_ms;
        self
    }
    
    pub fn with_constraint_count(mut self, count: u32) -> Self {
        self.constraint_count = count;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_fragment_proof_creation() {
        let proof = CompleteFragmentProof::new(
            1,
            vec![0; 512],
            ExecutionHash::from_bytes([1u8; 32]),
        );
        
        assert_eq!(proof.fragment_id, 1);
        assert_eq!(proof.proof_size, 512);
    }

    #[test]
    fn test_complete_fragment_proof_builder() {
        let proof = CompleteFragmentProof::new(
            1,
            vec![0; 512],
            ExecutionHash::from_bytes([1u8; 32]),
        )
        .with_input_commitments(vec![[1u8; 32]])
        .with_output_commitments(vec![[2u8; 32]])
        .with_generation_time(500)
        .with_constraint_count(1000);
        
        assert_eq!(proof.input_commitments.len(), 1);
        assert_eq!(proof.output_commitments.len(), 1);
        assert_eq!(proof.generation_time_ms, 500);
        assert_eq!(proof.constraint_count, 1000);
    }
}