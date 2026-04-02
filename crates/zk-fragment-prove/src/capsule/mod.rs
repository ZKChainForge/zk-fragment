//! Fragment Proof Capsule - self-contained proving unit

pub mod types;
pub mod builder;

pub use types::{
    FragmentMetadata, FragmentWitness, FragmentProofOutput,
    FragmentProof, FragmentProofCapsule,
};
pub use builder::CapsuleBuilder;