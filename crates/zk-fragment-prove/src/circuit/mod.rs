//! Circuit gadgets for fragment proving
//!
//! Components that would be implemented as Plonky2 gadgets

pub mod boundary_input;
pub mod boundary_output;
pub mod execution_hash;
pub mod fragment_wrapper;
pub mod merkle_membership;

pub use boundary_input::{BoundaryInputGadget, BoundaryInputConfig};
pub use boundary_output::{BoundaryOutputGadget, BoundaryOutputConfig};
pub use execution_hash::{ExecutionHashGadget, ExecutionHashConfig};
pub use fragment_wrapper::{FragmentCircuit, FragmentCircuitConfig, ConstraintBreakdown};
pub use merkle_membership::{MerkleMembershipGadget, MerkleMembershipConfig};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_circuit_integration() {
        use crate::capsule::FragmentMetadata;
        
        let metadata = FragmentMetadata {
            fragment_id: 1,
            constraint_count: 1000,
            input_boundary_count: 1,
            output_boundary_count: 1,
            execution_position: 1,
        };
        
        let config = FragmentCircuitConfig::new(metadata, 1000);
        let circuit = FragmentCircuit::new(config);
        
        let breakdown = circuit.constraint_breakdown();
        
        assert_eq!(breakdown.fragment_constraints, 1000);
        assert!(breakdown.total > 1000);
        assert!(breakdown.boundary_overhead() > 0.0);
    }
}