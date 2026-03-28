//! ZK-FRAGMENT Graph Analysis Library
//! 
//! This crate provides tools for analyzing ZK circuit constraint graphs
//! to identify optimal fragmentation strategies.
//! 
//! # Overview
//! 
//! The library treats ZK circuits as directed acyclic graphs (DAGs) where:
//! - Nodes represent constraints (gates)
//! - Edges represent data dependencies via wires
//! 
//! This graph structure enables:
//! - Finding natural fragmentation points (cut vertices)
//! - Computing valid execution orders (topological sort)
//! - Analyzing circuit structure for optimization
//! 
//! # Example
//! 
//! ```rust
//! use zk_fragment_graph::builder::GraphBuilder;
//! use zk_fragment_graph::algorithms;
//! 
//! // Build a simple circuit graph
//! let mut builder = GraphBuilder::new();
//! let x = builder.add_public_input();
//! let y = builder.add_public_input();
//! let sum = builder.add_addition(x, y);
//! builder.mark_public_output(sum);
//! let graph = builder.build();
//! 
//! // Analyze the graph
//! let order = algorithms::topological_sort(&graph).unwrap();
//! println!("Execution order: {:?}", order);
//! 
//! let stats = graph.compute_stats();
//! println!("Constraints: {}, Edges: {}", stats.constraint_count, stats.edge_count);
//! ```

pub mod types;
pub mod algorithms;
pub mod analysis;
pub mod builder;

// Re-export commonly used types
pub use types::{
    Constraint, ConstraintId, ConstraintType, ConstraintMetadata,
    Wire, WireId, WireRole,
    ConstraintGraph, Edge, GraphStats,
};

pub use builder::{GraphBuilder, InstrumentedCircuitBuilder};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::types::*;
    pub use crate::algorithms::*;
    pub use crate::analysis::*;
    pub use crate::builder::*;
}