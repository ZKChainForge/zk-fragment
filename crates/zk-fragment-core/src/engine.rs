use crate::fragment::{FragmentSpec, FragmentationResult, validate_fragmentation, compute_fragmentation_metrics, FragmentationMetrics};
use crate::strategy::{
    FragmentationStrategy, FragmentationConfig, FragmentationError,
    CutVertexStrategy, BalancedStrategy, MemoryAwareStrategy, StrategyType,
};
use crate::boundary::{detect_boundaries, BoundaryAnalysis, compute_boundary_stats, BoundaryStats};
use zk_fragment_graph::ConstraintGraph;

/// Main fragmentation engine
pub struct FragmentationEngine {
    config: FragmentationConfig,
}

impl FragmentationEngine {
    pub fn new(config: FragmentationConfig) -> Self {
        Self { config }
    }
    
    pub fn with_default_config() -> Self {
        Self::new(FragmentationConfig::default())
    }
    
    /// Fragment a constraint graph
    pub fn fragment(&self, graph: &ConstraintGraph) -> Result<FragmentationResult, FragmentationError> {
        // Validate config
        self.config.validate()
            .map_err(|e| FragmentationError::InvalidConfig(e))?;
        
        // Select strategy
        let strategy = self.select_strategy(graph);
        
        log::info!("Fragmenting with {} strategy", strategy.name());
        
        // Execute fragmentation
        let result = strategy.fragment(graph, &self.config)?;
        
        // Validate result
        let validation = validate_fragmentation(&result, graph);
        if !validation.is_valid {
            let error_msg = validation.errors.iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(FragmentationError::PartitionFailed(error_msg));
        }
        
        Ok(result)
    }
    
    /// Fragment and return with detailed analysis
    pub fn fragment_with_analysis(
        &self,
        graph: &ConstraintGraph,
    ) -> Result<FragmentationAnalysis, FragmentationError> {
        let result = self.fragment(graph)?;
        
        let boundary_analysis = detect_boundaries(graph, &result.fragments);
        let boundary_stats = compute_boundary_stats(
            &boundary_analysis,
            &result.fragments,
            graph.constraints.len(),
        );
        let metrics = compute_fragmentation_metrics(&result, graph);
        
        Ok(FragmentationAnalysis {
            result,
            boundary_analysis,
            boundary_stats,
            metrics,
        })
    }
    
    /// Select appropriate strategy based on graph properties
    fn select_strategy(&self, graph: &ConstraintGraph) -> Box<dyn FragmentationStrategy> {
        match self.config.strategy {
            StrategyType::CutVertex => Box::new(CutVertexStrategy::new()),
            StrategyType::Balanced => Box::new(BalancedStrategy::new()),
            StrategyType::MemoryAware => Box::new(MemoryAwareStrategy::new()),
            StrategyType::Auto => self.auto_select_strategy(graph),
        }
    }
    
    /// Automatically select best strategy
    fn auto_select_strategy(&self, graph: &ConstraintGraph) -> Box<dyn FragmentationStrategy> {
        // Check for natural cut points
        let cut_analysis = zk_fragment_graph::algorithms::find_cut_vertices(graph);
        
        if !cut_analysis.cut_vertices.is_empty() {
            log::info!("Auto-selected CutVertex strategy ({} cut vertices found)", 
                cut_analysis.cut_vertices.len());
            return Box::new(CutVertexStrategy::new());
        }
        
        // If memory limit specified, use memory-aware
        if self.config.max_memory_per_fragment.is_some() {
            log::info!("Auto-selected MemoryAware strategy (memory limit specified)");
            return Box::new(MemoryAwareStrategy::new());
        }
        
        // Default to balanced
        log::info!("Auto-selected Balanced strategy");
        Box::new(BalancedStrategy::new())
    }
}

/// Complete fragmentation analysis
#[derive(Debug)]
pub struct FragmentationAnalysis {
    pub result: FragmentationResult,
    pub boundary_analysis: BoundaryAnalysis,
    pub boundary_stats: BoundaryStats,
    pub metrics: FragmentationMetrics,
}

impl std::fmt::Display for FragmentationAnalysis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Fragmentation Analysis")?;
        writeln!(f, "======================")?;
        writeln!(f)?;
        writeln!(f, "Fragments: {}", self.result.fragments.len())?;
        writeln!(f, "Execution Order: {:?}", self.result.execution_order)?;
        writeln!(f, "Max Parallelism: {}", self.result.max_parallelism)?;
        writeln!(f, "Dependency Depth: {}", self.result.dependency_depth)?;
        writeln!(f)?;
        writeln!(f, "Boundary Statistics:")?;
        writeln!(f, "  Total Boundaries: {}", self.boundary_stats.total)?;
        writeln!(f, "  Avg per Fragment: {:.2}", self.boundary_stats.avg_per_fragment)?;
        writeln!(f, "  Max Output: {}", self.boundary_stats.max_output)?;
        writeln!(f, "  Max Input: {}", self.boundary_stats.max_input)?;
        writeln!(f, "  Overhead Ratio: {:.2}%", self.boundary_stats.overhead_ratio * 100.0)?;
        writeln!(f)?;
        writeln!(f, "{}", self.metrics)?;
        
        writeln!(f)?;
        writeln!(f, "Fragment Details:")?;
        for fragment in &self.result.fragments {
            writeln!(f, "  {}: {} constraints, {} in-boundaries, {} out-boundaries, deps: {:?}",
                fragment.id,
                fragment.constraint_count(),
                fragment.input_boundaries.len(),
                fragment.output_boundaries.len(),
                fragment.dependencies,
            )?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zk_fragment_graph::builder::{create_chain_circuit, create_diamond_circuit};

    #[test]
    fn test_engine_chain_circuit() {
        let graph = create_chain_circuit(50);
        let engine = FragmentationEngine::with_default_config();
        
        let result = engine.fragment(&graph);
        assert!(result.is_ok());
    }

    #[test]
    fn test_engine_with_analysis() {
        let graph = create_chain_circuit(100);
        let config = FragmentationConfig::with_fragment_count(4);
        let engine = FragmentationEngine::new(config);
        
        let analysis = engine.fragment_with_analysis(&graph);
        assert!(analysis.is_ok());
        
        let analysis = analysis.unwrap();
        println!("{}", analysis);
    }

    #[test]
    fn test_auto_strategy_selection() {
        let graph = create_diamond_circuit(4, 3);
        let engine = FragmentationEngine::with_default_config();
        
        let result = engine.fragment(&graph);
        assert!(result.is_ok());
    }
}