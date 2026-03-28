#[allow(unused_imports)]
use crate::types::{ConstraintGraph, ConstraintId, WireId};
use crate::algorithms;
use std::collections::HashMap;

/// Detailed metrics about a constraint graph
#[derive(Debug, Clone)]
pub struct DetailedMetrics {
    /// Basic graph statistics
    pub basic: BasicMetrics,
    /// Degree distribution
    pub degree_distribution: DegreeDistribution,
    /// Path metrics
    pub path_metrics: PathMetrics,
    /// Fragmentation suitability score
    pub fragmentation_score: f64,
}

#[derive(Debug, Clone)]
pub struct BasicMetrics {
    pub constraint_count: usize,
    pub wire_count: usize,
    pub edge_count: usize,
    pub source_count: usize,
    pub sink_count: usize,
    pub density: f64,
}

#[derive(Debug, Clone)]
pub struct DegreeDistribution {
    pub in_degrees: Vec<(usize, usize)>,  // (degree, count)
    pub out_degrees: Vec<(usize, usize)>, // (degree, count)
    pub average_in_degree: f64,
    pub average_out_degree: f64,
    pub max_in_degree: usize,
    pub max_out_degree: usize,
}

#[derive(Debug, Clone)]
pub struct PathMetrics {
    pub longest_path_length: usize,
    pub average_path_length: f64,
    pub depth_levels: usize,
    pub max_width: usize, // max nodes at any level
}

/// Compute detailed metrics for a constraint graph
pub fn compute_detailed_metrics(graph: &ConstraintGraph) -> DetailedMetrics {
    let basic = compute_basic_metrics(graph);
    let degree_distribution = compute_degree_distribution(graph);
    let path_metrics = compute_path_metrics(graph);
    let fragmentation_score = compute_fragmentation_score(graph, &basic, &degree_distribution);
    
    DetailedMetrics {
        basic,
        degree_distribution,
        path_metrics,
        fragmentation_score,
    }
}

fn compute_basic_metrics(graph: &ConstraintGraph) -> BasicMetrics {
    let constraint_count = graph.constraints.len();
    let wire_count = graph.wires.len();
    let edge_count = graph.edges.len();
    let source_count = graph.get_sources().len();
    let sink_count = graph.get_sinks().len();
    
    let possible_edges = if constraint_count > 1 {
        constraint_count * (constraint_count - 1)
    } else {
        1
    };
    let density = edge_count as f64 / possible_edges as f64;
    
    BasicMetrics {
        constraint_count,
        wire_count,
        edge_count,
        source_count,
        sink_count,
        density,
    }
}

fn compute_degree_distribution(graph: &ConstraintGraph) -> DegreeDistribution {
    let mut in_degree_counts: HashMap<usize, usize> = HashMap::new();
    let mut out_degree_counts: HashMap<usize, usize> = HashMap::new();
    
    let mut total_in = 0usize;
    let mut total_out = 0usize;
    let mut max_in = 0usize;
    let mut max_out = 0usize;
    
    for &id in graph.constraints.keys() {
        let in_deg = graph.in_degree(id);
        let out_deg = graph.out_degree(id);
        
        *in_degree_counts.entry(in_deg).or_insert(0) += 1;
        *out_degree_counts.entry(out_deg).or_insert(0) += 1;
        
        total_in += in_deg;
        total_out += out_deg;
        max_in = max_in.max(in_deg);
        max_out = max_out.max(out_deg);
    }
    
    let count = graph.constraints.len().max(1);
    let average_in_degree = total_in as f64 / count as f64;
    let average_out_degree = total_out as f64 / count as f64;
    
    let mut in_degrees: Vec<_> = in_degree_counts.into_iter().collect();
    let mut out_degrees: Vec<_> = out_degree_counts.into_iter().collect();
    in_degrees.sort_by_key(|(deg, _)| *deg);
    out_degrees.sort_by_key(|(deg, _)| *deg);
    
    DegreeDistribution {
        in_degrees,
        out_degrees,
        average_in_degree,
        average_out_degree,
        max_in_degree: max_in,
        max_out_degree: max_out,
    }
}

fn compute_path_metrics(graph: &ConstraintGraph) -> PathMetrics {
    if graph.constraints.is_empty() {
        return PathMetrics {
            longest_path_length: 0,
            average_path_length: 0.0,
            depth_levels: 0,
            max_width: 0,
        };
    }
    
    match algorithms::group_by_depth(graph) {
        Ok(levels) => {
            let depth_levels = levels.len();
            let max_width = levels.iter().map(|l| l.len()).max().unwrap_or(0);
            let longest_path_length = if depth_levels > 0 { depth_levels - 1 } else { 0 };
            
            // Compute average path length from sources to sinks
            let sources = graph.get_sources();
            let sinks = graph.get_sinks();
            
            let mut total_path_length = 0usize;
            let mut path_count = 0usize;
            
            for &source in &sources {
                for &sink in &sinks {
                    if let Some(path) = algorithms::shortest_path(graph, source, sink) {
                        total_path_length += path.len() - 1;
                        path_count += 1;
                    }
                }
            }
            
            let average_path_length = if path_count > 0 {
                total_path_length as f64 / path_count as f64
            } else {
                0.0
            };
            
            PathMetrics {
                longest_path_length,
                average_path_length,
                depth_levels,
                max_width,
            }
        }
        Err(_) => PathMetrics {
            longest_path_length: 0,
            average_path_length: 0.0,
            depth_levels: 0,
            max_width: 0,
        },
    }
}

fn compute_fragmentation_score(
    graph: &ConstraintGraph,
    basic: &BasicMetrics,
    degree_dist: &DegreeDistribution,
) -> f64 {
    if basic.constraint_count < 2 {
        return 0.0;
    }
    
    let mut score = 0.0;
    
    // Factor 1: Graph sparsity (sparser = better for fragmentation)
    // Score 0-25 based on density (lower density = higher score)
    let sparsity_score = (1.0 - basic.density.min(1.0)) * 25.0;
    score += sparsity_score;
    
    // Factor 2: Cut vertices available
    let cut_analysis = algorithms::find_cut_vertices(graph);
    let cut_vertex_ratio = cut_analysis.cut_vertices.len() as f64 / basic.constraint_count as f64;
    let cut_vertex_score = cut_vertex_ratio.min(0.5) * 50.0; // Max 25 points
    score += cut_vertex_score;
    
    // Factor 3: Low average degree (fewer wires per constraint = easier boundaries)
    let avg_degree = (degree_dist.average_in_degree + degree_dist.average_out_degree) / 2.0;
    let degree_score = if avg_degree < 5.0 {
        25.0 - (avg_degree * 5.0)
    } else {
        0.0
    };
    score += degree_score.max(0.0);
    
    // Factor 4: Multiple sources/sinks (natural parallelism)
    let parallelism_ratio = (basic.source_count + basic.sink_count) as f64 / basic.constraint_count as f64;
    let parallelism_score = parallelism_ratio.min(0.5) * 50.0; // Max 25 points
    score += parallelism_score;
    
    score.min(100.0)
}

/// Estimate memory requirements for proving the entire graph
pub fn estimate_memory_requirements(graph: &ConstraintGraph) -> MemoryEstimate {
    let constraint_count = graph.constraints.len();
    let wire_count = graph.wires.len();
    
    // Rough estimates based on typical ZK proving requirements
    // These are very approximate and depend heavily on the proof system
    
    // Per constraint: ~1KB for constraint data + ~4KB for proving
    let constraint_memory = constraint_count * 5 * 1024;
    
    // Per wire: ~256 bytes
    let wire_memory = wire_count * 256;
    
    // FFT/polynomial overhead: roughly 2x the base
    let fft_overhead = (constraint_memory + wire_memory) * 2;
    
    let total_bytes = constraint_memory + wire_memory + fft_overhead;
    
    MemoryEstimate {
        constraint_memory_bytes: constraint_memory,
        wire_memory_bytes: wire_memory,
        fft_overhead_bytes: fft_overhead,
        total_estimated_bytes: total_bytes,
        recommended_fragment_count: recommend_fragment_count(total_bytes),
    }
}

fn recommend_fragment_count(total_bytes: usize) -> usize {
    // Assume 8GB target per fragment
    let target_per_fragment = 8 * 1024 * 1024 * 1024; // 8GB
    let recommended = (total_bytes / target_per_fragment).max(1);
    
    // Round up to power of 2 for balanced tree aggregation
    recommended.next_power_of_two()
}

#[derive(Debug, Clone)]
pub struct MemoryEstimate {
    pub constraint_memory_bytes: usize,
    pub wire_memory_bytes: usize,
    pub fft_overhead_bytes: usize,
    pub total_estimated_bytes: usize,
    pub recommended_fragment_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn create_test_graph() -> ConstraintGraph {
        let mut graph = ConstraintGraph::new();
        
        for i in 0..5 {
            graph.add_wire(Wire::internal(WireId(i)));
        }
        
        for i in 0..5 {
            let inputs = if i > 0 { vec![WireId(i - 1)] } else { vec![] };
            let outputs = vec![WireId(i)];
            let c = Constraint::new(ConstraintId(i), ConstraintType::Add, inputs, outputs);
            graph.add_constraint(c);
        }
        
        for i in 0..5 {
            if i > 0 {
                graph.wires.get_mut(&WireId(i - 1)).unwrap().consumers.push(ConstraintId(i));
            }
            graph.wires.get_mut(&WireId(i)).unwrap().producer = Some(ConstraintId(i));
        }
        
        graph.build_edges_from_wires();
        graph
    }

    #[test]
    fn test_compute_detailed_metrics() {
        let graph = create_test_graph();
        let metrics = compute_detailed_metrics(&graph);
        
        assert_eq!(metrics.basic.constraint_count, 5);
        assert_eq!(metrics.basic.wire_count, 5);
        assert!(metrics.fragmentation_score >= 0.0);
        assert!(metrics.fragmentation_score <= 100.0);
    }

    #[test]
    fn test_memory_estimate() {
        let graph = create_test_graph();
        let estimate = estimate_memory_requirements(&graph);
        
        assert!(estimate.total_estimated_bytes > 0);
        assert!(estimate.recommended_fragment_count >= 1);
    }
}