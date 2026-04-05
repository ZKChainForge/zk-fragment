//! Parallel proving utilities

use crate::capsule::FragmentProofCapsule;
use anyhow::Result;
use std::sync::{Arc, Mutex};

/// Configuration for parallel proving
pub struct ParallelProverConfig {
    /// Number of parallel threads
    pub num_threads: usize,
    
    /// Timeout per fragment (seconds)
    pub timeout_per_fragment: u64,
}

impl Default for ParallelProverConfig {
    fn default() -> Self {
        ParallelProverConfig {
            num_threads: num_cpus::get(),
            timeout_per_fragment: 300,
        }
    }
}

/// Coordinator for parallel proving
pub struct ParallelProverCoordinator {
    config: ParallelProverConfig,
}

impl ParallelProverCoordinator {
    pub fn new(config: ParallelProverConfig) -> Self {
        ParallelProverCoordinator { config }
    }
    
    /// Estimate time for parallel proving
    pub fn estimate_parallel_time(
        capsule_count: usize,
        average_time_per_fragment: u64,
    ) -> u64 {
        let num_threads = num_cpus::get();
        let batches = (capsule_count + num_threads - 1) / num_threads;
        batches as u64 * average_time_per_fragment
    }
    
    /// Calculate speedup vs sequential
    pub fn calculate_speedup(
        sequential_time: u64,
        parallel_time: u64,
    ) -> f64 {
        if parallel_time == 0 {
            return 1.0;
        }
        sequential_time as f64 / parallel_time as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_prover_creation() {
        let config = ParallelProverConfig::default();
        let _coordinator = ParallelProverCoordinator::new(config);
    }

    #[test]
    fn test_parallel_time_estimation() {
        let time = ParallelProverCoordinator::estimate_parallel_time(8, 1000);
        assert!(time > 0);
    }

    #[test]
    fn test_speedup_calculation() {
        let speedup = ParallelProverCoordinator::calculate_speedup(4000, 1000);
        assert_eq!(speedup, 4.0);
    }
}