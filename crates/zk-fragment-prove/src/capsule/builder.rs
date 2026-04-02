//! Build Fragment Proof Capsules from fragment specifications

use super::types::*;
use crate::commitment::{CommitmentConfig, CommitmentGenerator};
use zk_fragment_core::fragment::spec::FragmentSpec;
use anyhow::Result;

/// Builds Fragment Proof Capsules
pub struct CapsuleBuilder {
    /// Configuration for commitment generation
    commitment_config: CommitmentConfig,
}

impl CapsuleBuilder {
    /// Create a new capsule builder
    pub fn new(shared_secret: [u8; 32]) -> Self {
        CapsuleBuilder {
            commitment_config: CommitmentConfig {
                fragment_id: 0,
                shared_secret,
                include_openings: false,
            },
        }
    }
    
    /// Build a capsule from a fragment specification
    pub fn build_capsule(
        &mut self,
        fragment_id: u32,
        spec: &FragmentSpec,
        execution_position: u32,
    ) -> Result<FragmentProofCapsule> {
        // Update fragment ID in config
        self.commitment_config.fragment_id = fragment_id;
        
        // Create metadata
        let metadata = FragmentMetadata {
            fragment_id,
            constraint_count: spec.constraints.len() as u32,
            input_boundary_count: spec
                .boundaries
                .iter()
                .filter(|b| b.source_fragment != fragment_id)
                .count() as u32,
            output_boundary_count: spec
                .boundaries
                .iter()
                .filter(|b| b.source_fragment == fragment_id)
                .count() as u32,
            execution_position,
        };
        
        // Create capsule
        let capsule = FragmentProofCapsule::new(metadata);
        
        Ok(capsule)
    }
    
    /// Build multiple capsules
    pub fn build_capsules(
        &mut self,
        specs: &[(u32, FragmentSpec)], // (fragment_id, spec) pairs
    ) -> Result<Vec<FragmentProofCapsule>> {
        specs
            .iter()
            .enumerate()
            .map(|(pos, (frag_id, spec))| {
                self.build_capsule(*frag_id, spec, pos as u32)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capsule_builder_creation() {
        let builder = CapsuleBuilder::new([1u8; 32]);
        assert_eq!(builder.commitment_config.fragment_id, 0);
    }
}