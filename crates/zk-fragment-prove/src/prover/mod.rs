//! Fragment proving infrastructure

pub mod fragment_prover;
pub mod parallel;
pub mod checkpoint;

pub use fragment_prover::{FragmentProver, FragmentProverConfig};
pub use parallel::{ParallelProverCoordinator, ParallelProverConfig};
pub use checkpoint::{ProvingCheckpoint, CheckpointManager};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prover_infrastructure() {
        let config = FragmentProverConfig::default();
        let _prover = FragmentProver::new(config);
        
        let parallel_config = ParallelProverConfig::default();
        let _parallel = ParallelProverCoordinator::new(parallel_config);
        
        let checkpoint = ProvingCheckpoint::new("test".to_string(), 5);
        assert_eq!(checkpoint.total_capsules, 5);
    }
}