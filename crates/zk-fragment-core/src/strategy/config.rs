use serde::{Deserialize, Serialize};

/// Strategy selection for fragmentation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategyType {
    /// Use cut vertices for natural fragmentation
    CutVertex,
    /// Balanced K-way partitioning
    Balanced,
    /// Memory-aware fragmentation
    MemoryAware,
    /// Automatic selection based on graph properties
    Auto,
}

impl Default for StrategyType {
    fn default() -> Self {
        StrategyType::Auto
    }
}

/// Configuration for the fragmentation process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentationConfig {
    /// Strategy to use
    pub strategy: StrategyType,
    
    /// Target number of fragments (None for automatic)
    pub target_fragment_count: Option<usize>,
    
    /// Minimum constraints per fragment
    pub min_fragment_size: usize,
    
    /// Maximum constraints per fragment (None for no limit)
    pub max_fragment_size: Option<usize>,
    
    /// Maximum memory per fragment in bytes (for MemoryAware strategy)
    pub max_memory_per_fragment: Option<u64>,
    
    /// Whether to optimize for parallelism
    pub optimize_parallelism: bool,
    
    /// Whether to minimize boundaries (may reduce parallelism)
    pub minimize_boundaries: bool,
    
    /// Maximum allowed boundary overhead ratio
    pub max_boundary_overhead: f64,
}

impl Default for FragmentationConfig {
    fn default() -> Self {
        Self {
            strategy: StrategyType::Auto,
            target_fragment_count: None,
            min_fragment_size: 10,
            max_fragment_size: None,
            max_memory_per_fragment: None,
            optimize_parallelism: true,
            minimize_boundaries: false,
            max_boundary_overhead: 0.5,
        }
    }
}

impl FragmentationConfig {
    /// Create config for a specific number of fragments
    pub fn with_fragment_count(count: usize) -> Self {
        Self {
            target_fragment_count: Some(count),
            ..Default::default()
        }
    }
    
    /// Create config for memory-aware fragmentation
    pub fn memory_aware(max_memory_bytes: u64) -> Self {
        Self {
            strategy: StrategyType::MemoryAware,
            max_memory_per_fragment: Some(max_memory_bytes),
            ..Default::default()
        }
    }
    
    /// Create config prioritizing parallelism
    pub fn max_parallelism() -> Self {
        Self {
            optimize_parallelism: true,
            minimize_boundaries: false,
            ..Default::default()
        }
    }
    
    /// Create config minimizing boundary overhead
    pub fn min_boundaries() -> Self {
        Self {
            minimize_boundaries: true,
            optimize_parallelism: false,
            ..Default::default()
        }
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if let Some(count) = self.target_fragment_count {
            if count == 0 {
                return Err("Target fragment count must be at least 1".to_string());
            }
        }
        
        if self.min_fragment_size == 0 {
            return Err("Minimum fragment size must be at least 1".to_string());
        }
        
        if let Some(max_size) = self.max_fragment_size {
            if max_size < self.min_fragment_size {
                return Err("Maximum fragment size must be >= minimum".to_string());
            }
        }
        
        if self.max_boundary_overhead < 0.0 || self.max_boundary_overhead > 10.0 {
            return Err("Boundary overhead must be between 0 and 10".to_string());
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = FragmentationConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_with_fragment_count() {
        let config = FragmentationConfig::with_fragment_count(4);
        assert_eq!(config.target_fragment_count, Some(4));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_config() {
        let mut config = FragmentationConfig::default();
        config.min_fragment_size = 0;
        assert!(config.validate().is_err());
    }
}