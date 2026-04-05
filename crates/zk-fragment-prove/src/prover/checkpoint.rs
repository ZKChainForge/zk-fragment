//! Checkpoint system for fault recovery

use crate::capsule::FragmentProofCapsule;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use std::path::Path;

/// Checkpoint of proving progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvingCheckpoint {
    /// Checkpoint ID
    pub checkpoint_id: String,
    
    /// Capsules completed
    pub completed_capsules: usize,
    
    /// Total capsules
    pub total_capsules: usize,
    
    /// Timestamp
    pub timestamp: u64,
    
    /// Elapsed time (ms)
    pub elapsed_ms: u64,
}

impl ProvingCheckpoint {
    pub fn new(checkpoint_id: String, total: usize) -> Self {
        ProvingCheckpoint {
            checkpoint_id,
            completed_capsules: 0,
            total_capsules: total,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            elapsed_ms: 0,
        }
    }
    
    /// Get progress percentage
    pub fn progress_percentage(&self) -> f64 {
        if self.total_capsules == 0 {
            return 100.0;
        }
        (self.completed_capsules as f64 / self.total_capsules as f64) * 100.0
    }
    
    /// Estimate remaining time
    pub fn estimate_remaining_ms(&self) -> u64 {
        if self.completed_capsules == 0 {
            return 0;
        }
        let rate = self.elapsed_ms as f64 / self.completed_capsules as f64;
        let remaining = self.total_capsules - self.completed_capsules;
        (rate * remaining as f64) as u64
    }
}

/// Checkpoint manager
pub struct CheckpointManager;

impl CheckpointManager {
    /// Save checkpoint to file
    pub fn save_checkpoint(
        checkpoint: &ProvingCheckpoint,
        path: &Path,
    ) -> Result<()> {
        let json = serde_json::to_string_pretty(checkpoint)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    /// Load checkpoint from file
    pub fn load_checkpoint(path: &Path) -> Result<ProvingCheckpoint> {
        let json = std::fs::read_to_string(path)?;
        let checkpoint = serde_json::from_str(&json)?;
        Ok(checkpoint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkpoint_creation() {
        let checkpoint = ProvingCheckpoint::new("test_1".to_string(), 10);
        assert_eq!(checkpoint.completed_capsules, 0);
        assert_eq!(checkpoint.total_capsules, 10);
    }

    #[test]
    fn test_checkpoint_progress() {
        let mut checkpoint = ProvingCheckpoint::new("test_1".to_string(), 10);
        checkpoint.completed_capsules = 5;
        
        assert_eq!(checkpoint.progress_percentage(), 50.0);
    }

    #[test]
    fn test_checkpoint_remaining_time() {
        let mut checkpoint = ProvingCheckpoint::new("test_1".to_string(), 10);
        checkpoint.completed_capsules = 2;
        checkpoint.elapsed_ms = 2000;
        
        let remaining = checkpoint.estimate_remaining_ms();
        assert!(remaining > 0);
    }
}