use crate::fragment::{FragmentId, BoundaryWire};
use zk_fragment_graph::WireId;
use serde::{Deserialize, Serialize};

/// Type of commitment scheme used for boundaries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitmentScheme {
    /// Poseidon hash commitment
    Poseidon,
    /// Pedersen commitment
    Pedersen,
    /// Simple hash (less secure but faster)
    SimpleHash,
}

impl Default for CommitmentScheme {
    fn default() -> Self {
        CommitmentScheme::Poseidon
    }
}

/// Specification for a boundary commitment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryCommitmentSpec {
    /// The boundary wire
    pub wire_id: WireId,
    /// Source fragment
    pub source_fragment: FragmentId,
    /// Target fragment(s)
    pub target_fragments: Vec<FragmentId>,
    /// Commitment scheme to use
    pub scheme: CommitmentScheme,
    /// Index in the source fragment's commitment list
    pub commitment_index: usize,
}

/// Configuration for boundary commitments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryCommitmentConfig {
    /// Default commitment scheme
    pub default_scheme: CommitmentScheme,
    /// Whether to use deterministic blinding
    pub deterministic_blinding: bool,
    /// Salt for deterministic blinding derivation
    pub blinding_salt: Option<Vec<u8>>,
}

impl Default for BoundaryCommitmentConfig {
    fn default() -> Self {
        Self {
            default_scheme: CommitmentScheme::Poseidon,
            deterministic_blinding: true,
            blinding_salt: None,
        }
    }
}

/// Generate commitment specifications for all boundaries
pub fn generate_commitment_specs(
    boundaries: &[BoundaryWire],
    config: &BoundaryCommitmentConfig,
) -> Vec<BoundaryCommitmentSpec> {
    use std::collections::HashMap;
    
    // Group boundaries by (source_fragment, wire_id) to handle multi-target
    let mut grouped: HashMap<(FragmentId, WireId), Vec<FragmentId>> = HashMap::new();
    
    for boundary in boundaries {
        grouped
            .entry((boundary.source_fragment, boundary.wire_id))
            .or_default()
            .push(boundary.target_fragment);
    }
    
    let mut specs = Vec::new();
    let mut indices: HashMap<FragmentId, usize> = HashMap::new();
    
    for ((source, wire_id), targets) in grouped {
        let index = *indices.entry(source).or_insert(0);
        
        specs.push(BoundaryCommitmentSpec {
            wire_id,
            source_fragment: source,
            target_fragments: targets,
            scheme: config.default_scheme,
            commitment_index: index,
        });
        
        *indices.get_mut(&source).unwrap() += 1;
    }
    
    specs
}

/// Commitment data for a boundary wire (used during proving)
#[derive(Debug, Clone)]
pub struct BoundaryCommitment {
    /// The committed value (as bytes)
    pub value_bytes: Vec<u8>,
    /// The commitment (hash/commitment output)
    pub commitment: Vec<u8>,
    /// Blinding factor (if applicable)
    pub blinding: Option<Vec<u8>>,
    /// Wire ID
    pub wire_id: WireId,
}

/// Generate a Poseidon commitment (placeholder - actual implementation needs Poseidon)
pub fn compute_poseidon_commitment(value: &[u8], salt: &[u8]) -> Vec<u8> {
    // Placeholder: in real implementation, use Plonky2's Poseidon
    // For now, use a simple hash
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    salt.hash(&mut hasher);
    
    hasher.finish().to_le_bytes().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_commitment_specs() {
        let boundaries = vec![
            BoundaryWire {
                wire_id: WireId(0),
                source_fragment: FragmentId(0),
                target_fragment: FragmentId(1),
                source_index: 0,
                target_index: 0,
            },
            BoundaryWire {
                wire_id: WireId(1),
                source_fragment: FragmentId(0),
                target_fragment: FragmentId(2),
                source_index: 1,
                target_index: 0,
            },
        ];
        
        let config = BoundaryCommitmentConfig::default();
        let specs = generate_commitment_specs(&boundaries, &config);
        
        assert_eq!(specs.len(), 2);
    }
}