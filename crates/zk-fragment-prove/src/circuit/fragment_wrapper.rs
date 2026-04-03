//! Wrapper circuit for complete fragment proof
//!
//! Combines all gadgets into one complete circuit

use super::boundary_input::{BoundaryInputGadget, BoundaryInputConfig};
use super::boundary_output::{BoundaryOutputGadget, BoundaryOutputConfig};
use super::execution_hash::{ExecutionHashGadget, ExecutionHashConfig};
use crate::execution_hash::ExecutionHash;
use crate::capsule::FragmentMetadata;
use serde::{Serialize, Deserialize};
use anyhow::Result;

/// Complete fragment circuit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentCircuitConfig {
    /// Fragment metadata
    pub metadata: FragmentMetadata,
    
    /// Fragment constraints
    pub fragment_constraints: u32,
    
    /// Shared secret
    pub shared_secret: [u8; 32],
}

impl FragmentCircuitConfig {
    pub fn new(metadata: FragmentMetadata, fragment_constraints: u32) -> Self {
        FragmentCircuitConfig {
            metadata,
            fragment_constraints,
            shared_secret: [0u8; 32],
        }
    }
    
    pub fn with_secret(mut self, secret: [u8; 32]) -> Self {
        self.shared_secret = secret;
        self
    }
}

/// Complete fragment circuit
pub struct FragmentCircuit {
    config: FragmentCircuitConfig,
    input_gadget: BoundaryInputGadget,
    output_gadget: BoundaryOutputGadget,
    hash_gadget: ExecutionHashGadget,
}

impl FragmentCircuit {
    pub fn new(config: FragmentCircuitConfig) -> Self {
        let metadata = &config.metadata;
        
        let input_config = BoundaryInputConfig::new(metadata.input_boundary_count as usize);
        let output_config = BoundaryOutputConfig::new(
            metadata.fragment_id,
            config.shared_secret,
            metadata.output_boundary_count as usize,
        );
        let hash_config = ExecutionHashConfig::new(
            metadata.fragment_id,
            metadata.input_boundary_count as usize,
            metadata.output_boundary_count as usize,
        );
        
        FragmentCircuit {
            config,
            input_gadget: BoundaryInputGadget::new(input_config),
            output_gadget: BoundaryOutputGadget::new(output_config),
            hash_gadget: ExecutionHashGadget::new(hash_config),
        }
    }
    
    /// Calculate total circuit constraints
    pub fn total_constraints(&self) -> u32 {
        self.config.fragment_constraints
            + self.input_gadget.constraint_count()
            + self.output_gadget.constraint_count()
            + self.hash_gadget.constraint_count()
    }
    
    /// Get circuit components breakdown
    pub fn constraint_breakdown(&self) -> ConstraintBreakdown {
        ConstraintBreakdown {
            fragment_constraints: self.config.fragment_constraints,
            input_boundary_constraints: self.input_gadget.constraint_count(),
            output_boundary_constraints: self.output_gadget.constraint_count(),
            execution_hash_constraints: self.hash_gadget.constraint_count(),
            total: self.total_constraints(),
        }
    }
    
    /// Get metadata reference
    pub fn metadata(&self) -> &FragmentMetadata {
        &self.config.metadata
    }
}

/// Breakdown of constraints by component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintBreakdown {
    pub fragment_constraints: u32,
    pub input_boundary_constraints: u32,
    pub output_boundary_constraints: u32,
    pub execution_hash_constraints: u32,
    pub total: u32,
}

impl ConstraintBreakdown {
    pub fn boundary_overhead(&self) -> f64 {
        if self.fragment_constraints == 0 {
            return 0.0;
        }
        let boundary_total = self.input_boundary_constraints
            + self.output_boundary_constraints
            + self.execution_hash_constraints;
        (boundary_total as f64 / self.fragment_constraints as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fragment_circuit_creation() {
        let metadata = FragmentMetadata {
            fragment_id: 0,
            constraint_count: 1000,
            input_boundary_count: 0,
            output_boundary_count: 1,
            execution_position: 0,
        };
        
        let config = FragmentCircuitConfig::new(metadata, 1000);
        let circuit = FragmentCircuit::new(config);
        
        assert_eq!(circuit.metadata().fragment_id, 0);
        assert!(circuit.total_constraints() > 1000);
    }

    #[test]
    fn test_fragment_circuit_constraints() {
        let metadata = FragmentMetadata {
            fragment_id: 1,
            constraint_count: 500,
            input_boundary_count: 1,
            output_boundary_count: 1,
            execution_position: 1,
        };
        
        let config = FragmentCircuitConfig::new(metadata, 500);
        let circuit = FragmentCircuit::new(config);
        
        let breakdown = circuit.constraint_breakdown();
        
        assert_eq!(breakdown.fragment_constraints, 500);
        assert!(breakdown.input_boundary_constraints > 0);
        assert!(breakdown.output_boundary_constraints > 0);
        assert!(breakdown.execution_hash_constraints > 0);
        assert_eq!(breakdown.total, circuit.total_constraints());
    }

    #[test]
    fn test_fragment_circuit_overhead() {
        let metadata = FragmentMetadata {
            fragment_id: 0,
            constraint_count: 10000,
            input_boundary_count: 2,
            output_boundary_count: 2,
            execution_position: 0,
        };
        
        let config = FragmentCircuitConfig::new(metadata, 10000);
        let circuit = FragmentCircuit::new(config);
        
        let breakdown = circuit.constraint_breakdown();
        let overhead = breakdown.boundary_overhead();
        
        assert!(overhead > 0.0);
        assert!(overhead < 100.0);
    }

    #[test]
    fn test_fragment_circuit_no_boundaries() {
        let metadata = FragmentMetadata {
            fragment_id: 0,
            constraint_count: 1000,
            input_boundary_count: 0,
            output_boundary_count: 0,
            execution_position: 0,
        };
        
        let config = FragmentCircuitConfig::new(metadata, 1000);
        let circuit = FragmentCircuit::new(config);
        
        let breakdown = circuit.constraint_breakdown();
        
        assert_eq!(breakdown.input_boundary_constraints, 0);
        assert_eq!(breakdown.output_boundary_constraints, 0);
    }

    #[test]
    fn test_fragment_circuit_many_boundaries() {
        let metadata = FragmentMetadata {
            fragment_id: 1,
            constraint_count: 100,
            input_boundary_count: 10,
            output_boundary_count: 10,
            execution_position: 1,
        };
        
        let config = FragmentCircuitConfig::new(metadata, 100);
        let circuit = FragmentCircuit::new(config);
        
        let breakdown = circuit.constraint_breakdown();
        
        assert!(breakdown.input_boundary_constraints > 0);
        assert!(breakdown.output_boundary_constraints > 0);
        let overhead = breakdown.boundary_overhead();
        assert!(overhead > 100.0); // More boundaries than fragment constraints
    }
}