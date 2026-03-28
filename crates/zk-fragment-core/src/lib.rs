//! ZK-FRAGMENT Core Library
//! 
//! This crate provides the core fragmentation engine for ZK circuits.
//! 
//! # Overview
//! 
//! The library takes a constraint graph and fragments it into independent
//! pieces that can be proven separately and aggregated.
//! 
//! # Example
//! 
//! ```rust
//! use zk_fragment_graph::builder::create_chain_circuit;
//! use zk_fragment_core::{FragmentationEngine, FragmentationConfig};
//! 
//! // Create a test circuit
//! let graph = create_chain_circuit(100);
//! 
//! // Configure fragmentation
//! let config = FragmentationConfig::with_fragment_count(4);
//! let engine = FragmentationEngine::new(config);
//! 
//! // Fragment the circuit
//! let result = engine.fragment(&graph).unwrap();
//! 
//! println!("Created {} fragments", result.fragments.len());
//! println!("Total boundaries: {}", result.total_boundary_wires);
//! ```

pub mod fragment;
pub mod strategy;
pub mod boundary;
pub mod witness;
pub mod engine;

// Re-export main types
pub use fragment::{
    FragmentId, FragmentSpec, BoundaryWire, FragmentationResult,
    FragmentValidationError, FragmentValidationResult, FragmentationMetrics,
};

pub use strategy::{
    FragmentationStrategy, FragmentationConfig, FragmentationError, StrategyType,
    CutVertexStrategy, BalancedStrategy, MemoryAwareStrategy,
};

pub use boundary::{
    BoundaryAnalysis, BoundaryStats, BoundaryCommitmentSpec, CommitmentScheme,
    WireRoutingTable, WireRoute,
};

pub use witness::{FragmentWitness, WitnessPartitioner, WitnessMapper};

pub use engine::{FragmentationEngine, FragmentationAnalysis};

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::fragment::*;
    pub use crate::strategy::*;
    pub use crate::boundary::*;
    pub use crate::witness::*;
    pub use crate::engine::*;
}