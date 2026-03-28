use super::{FragmentSpec, FragmentationResult};
use zk_fragment_graph::ConstraintGraph;

/// Metrics for evaluating fragmentation quality
#[derive(Debug, Clone)]
pub struct FragmentationMetrics {
    /// Number of fragments
    pub fragment_count: usize,
    /// Total constraints across all fragments
    pub total_constraints: usize,
    /// Total boundary wires
    pub total_boundaries: usize,
    /// Average fragment size
    pub avg_fragment_size: f64,
    /// Size of largest fragment
    pub max_fragment_size: usize,
    /// Size of smallest fragment
    pub min_fragment_size: usize,
    /// Standard deviation of fragment sizes
    pub size_std_dev: f64,
    /// Balance score (0-1, higher is more balanced)
    pub balance_score: f64,
    /// Boundary overhead ratio (boundaries / constraints)
    pub boundary_overhead: f64,
    /// Maximum parallelism possible
    pub max_parallelism: usize,
    /// Depth of dependency chain
    pub dependency_depth: usize,
    /// Estimated proving speedup vs monolithic
    pub estimated_speedup: f64,
}

/// Compute metrics for a fragmentation result
pub fn compute_fragmentation_metrics(
    result: &FragmentationResult,
    _graph: &ConstraintGraph,
) -> FragmentationMetrics {
    let fragment_count = result.fragments.len();
    
    if fragment_count == 0 {
        return FragmentationMetrics {
            fragment_count: 0,
            total_constraints: 0,
            total_boundaries: 0,
            avg_fragment_size: 0.0,
            max_fragment_size: 0,
            min_fragment_size: 0,
            size_std_dev: 0.0,
            balance_score: 1.0,
            boundary_overhead: 0.0,
            max_parallelism: 0,
            dependency_depth: 0,
            estimated_speedup: 1.0,
        };
    }
    
    let sizes: Vec<usize> = result.fragments.iter()
        .map(|f| f.constraint_count())
        .collect();
    
    let total_constraints: usize = sizes.iter().sum();
    let total_boundaries = result.total_boundary_wires;
    
    let avg_fragment_size = total_constraints as f64 / fragment_count as f64;
    let max_fragment_size = *sizes.iter().max().unwrap_or(&0);
    let min_fragment_size = *sizes.iter().min().unwrap_or(&0);
    
    // Compute standard deviation
    let variance: f64 = sizes.iter()
        .map(|&s| (s as f64 - avg_fragment_size).powi(2))
        .sum::<f64>() / fragment_count as f64;
    let size_std_dev = variance.sqrt();
    
    // Balance score: 1 - (std_dev / avg)
    // Perfect balance = 1.0, highly unbalanced approaches 0
    let balance_score = if avg_fragment_size > 0.0 {
        (1.0 - (size_std_dev / avg_fragment_size)).max(0.0)
    } else {
        1.0
    };
    
    // Boundary overhead
    let boundary_overhead = if total_constraints > 0 {
        total_boundaries as f64 / total_constraints as f64
    } else {
        0.0
    };
    
    // Estimated speedup
    // Simplified model: speedup from parallelism minus boundary overhead
    let parallel_speedup = result.max_parallelism as f64;
    let overhead_factor = 1.0 + boundary_overhead * 0.5; // 50% overhead per boundary ratio
    let estimated_speedup = (parallel_speedup / overhead_factor).max(1.0);
    
    FragmentationMetrics {
        fragment_count,
        total_constraints,
        total_boundaries,
        avg_fragment_size,
        max_fragment_size,
        min_fragment_size,
        size_std_dev,
        balance_score,
        boundary_overhead,
        max_parallelism: result.max_parallelism,
        dependency_depth: result.dependency_depth,
        estimated_speedup,
    }
}

impl std::fmt::Display for FragmentationMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Fragmentation Metrics:")?;
        writeln!(f, "  Fragments: {}", self.fragment_count)?;
        writeln!(f, "  Total Constraints: {}", self.total_constraints)?;
        writeln!(f, "  Total Boundaries: {}", self.total_boundaries)?;
        writeln!(f, "  Avg Fragment Size: {:.1}", self.avg_fragment_size)?;
        writeln!(f, "  Fragment Size Range: {} - {}", self.min_fragment_size, self.max_fragment_size)?;
        writeln!(f, "  Size Std Dev: {:.2}", self.size_std_dev)?;
        writeln!(f, "  Balance Score: {:.2}", self.balance_score)?;
        writeln!(f, "  Boundary Overhead: {:.2}%", self.boundary_overhead * 100.0)?;
        writeln!(f, "  Max Parallelism: {}", self.max_parallelism)?;
        writeln!(f, "  Dependency Depth: {}", self.dependency_depth)?;
        writeln!(f, "  Estimated Speedup: {:.2}x", self.estimated_speedup)?;
        Ok(())
    }
}

/// Compare two fragmentation results
pub fn compare_fragmentations(
    a: &FragmentationMetrics,
    b: &FragmentationMetrics,
) -> FragmentationComparison {
    FragmentationComparison {
        balance_diff: b.balance_score - a.balance_score,
        boundary_diff: b.boundary_overhead - a.boundary_overhead,
        parallelism_diff: b.max_parallelism as i32 - a.max_parallelism as i32,
        speedup_diff: b.estimated_speedup - a.estimated_speedup,
        better: if b.estimated_speedup > a.estimated_speedup { "B" } else { "A" }.to_string(),
    }
}

#[derive(Debug)]
pub struct FragmentationComparison {
    pub balance_diff: f64,
    pub boundary_diff: f64,
    pub parallelism_diff: i32,
    pub speedup_diff: f64,
    pub better: String,
}