//! Extract fragment witness from full witness
//!
//! Maps full circuit witness to individual fragment witness subsets

use crate::capsule::types::{FragmentWitness, FragmentMetadata};
use zk_fragment_core::witness::mapper::WitnessIndexMap;
use anyhow::{Result, bail};
use serde::{Serialize, Deserialize};

/// Witness extraction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionConfig {
    /// Include boundary values
    pub include_boundaries: bool,
    
    /// Include public inputs
    pub include_public_inputs: bool,
    
    /// Strict validation
    pub validate_indices: bool,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        ExtractionConfig {
            include_boundaries: true,
            include_public_inputs: true,
            validate_indices: true,
        }
    }
}

/// Witness extractor for fragments
pub struct WitnessExtractor {
    config: ExtractionConfig,
}

impl WitnessExtractor {
    /// Create new extractor
    pub fn new(config: ExtractionConfig) -> Self {
        WitnessExtractor { config }
    }
    
    /// Extract fragment witness from full witness
    ///
    /// Steps:
    /// 1. Identify which witness values belong to this fragment
    /// 2. Extract local witness values
    /// 3. Extract public input values
    /// 4. Return as FragmentWitness
    pub fn extract(
        &self,
        full_witness: &[u64],
        fragment_id: u32,
        metadata: &FragmentMetadata,
        index_map: &WitnessIndexMap,
    ) -> Result<FragmentWitness> {
        // Step 1: Get indices for this fragment
        let fragment_indices = index_map.get_fragment_indices(fragment_id)?;
        
        // Validate
        if self.config.validate_indices {
            self.validate_indices(&fragment_indices, full_witness.len())?;
        }
        
        // Step 2: Extract local witness
        let local_witness = self.extract_values(
            full_witness,
            &fragment_indices.local_witness_indices,
        )?;
        
        // Step 3: Extract public inputs
        let public_inputs = if self.config.include_public_inputs {
            self.extract_values(
                full_witness,
                &fragment_indices.public_input_indices,
            )?
        } else {
            Vec::new()
        };
        
        // Step 4: Create FragmentWitness
        let witness = FragmentWitness::new()
            .with_local_witness(local_witness)
            .with_public_inputs(public_inputs);
        
        Ok(witness)
    }
    
    /// Extract multiple fragment witnesses
    pub fn extract_all(
        &self,
        full_witness: &[u64],
        metadatas: &[FragmentMetadata],
        index_map: &WitnessIndexMap,
    ) -> Result<Vec<FragmentWitness>> {
        metadatas
            .iter()
            .map(|metadata| {
                self.extract(
                    full_witness,
                    metadata.fragment_id,
                    metadata,
                    index_map,
                )
            })
            .collect()
    }
    
    /// Extract values at given indices
    fn extract_values(
        &self,
        full_witness: &[u64],
        indices: &[usize],
    ) -> Result<Vec<u64>> {
        let mut values = Vec::with_capacity(indices.len());
        
        for &idx in indices {
            if idx >= full_witness.len() {
                bail!("Index {} out of bounds (witness size: {})", idx, full_witness.len());
            }
            values.push(full_witness[idx]);
        }
        
        Ok(values)
    }
    
    /// Validate indices are valid
    fn validate_indices(
        &self,
        fragment_indices: &FragmentIndices,
        witness_size: usize,
    ) -> Result<()> {
        // Check all indices are within bounds
        for &idx in &fragment_indices.local_witness_indices {
            if idx >= witness_size {
                bail!("Local witness index {} out of bounds", idx);
            }
        }
        
        for &idx in &fragment_indices.public_input_indices {
            if idx >= witness_size {
                bail!("Public input index {} out of bounds", idx);
            }
        }
        
        // Check no overlap (optional strict check)
        let mut seen = std::collections::HashSet::new();
        for &idx in &fragment_indices.local_witness_indices {
            if !seen.insert(idx) {
                bail!("Duplicate local witness index: {}", idx);
            }
        }
        
        Ok(())
    }
}

/// Fragment indices from map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentIndices {
    /// Indices for local witness
    pub local_witness_indices: Vec<usize>,
    
    /// Indices for public inputs
    pub public_input_indices: Vec<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_map() -> WitnessIndexMap {
        WitnessIndexMap::new()
    }

    #[test]
    fn test_extractor_creation() {
        let config = ExtractionConfig::default();
        let _extractor = WitnessExtractor::new(config);
    }

    #[test]
    fn test_extract_values() {
        let config = ExtractionConfig::default();
        let extractor = WitnessExtractor::new(config);
        
        let witness = vec![1, 2, 3, 4, 5];
        let indices = vec![0, 2, 4];
        
        let values = extractor.extract_values(&witness, &indices).unwrap();
        assert_eq!(values, vec![1, 3, 5]);
    }

    #[test]
    fn test_extract_values_out_of_bounds() {
        let config = ExtractionConfig::default();
        let extractor = WitnessExtractor::new(config);
        
        let witness = vec![1, 2, 3];
        let indices = vec![0, 5]; // Index 5 is out of bounds
        
        assert!(extractor.extract_values(&witness, &indices).is_err());
    }

    #[test]
    fn test_extract_values_empty() {
        let config = ExtractionConfig::default();
        let extractor = WitnessExtractor::new(config);
        
        let witness = vec![1, 2, 3];
        let indices = vec![];
        
        let values = extractor.extract_values(&witness, &indices).unwrap();
        assert_eq!(values.len(), 0);
    }
}