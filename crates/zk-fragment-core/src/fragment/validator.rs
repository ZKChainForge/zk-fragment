use super::{FragmentSpec, FragmentId, FragmentationResult};
use zk_fragment_graph::{ConstraintGraph, ConstraintId};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FragmentValidationError {
    #[error("Constraint {0} appears in multiple fragments")]
    DuplicateConstraint(ConstraintId),
    
    #[error("Constraint {0} not assigned to any fragment")]
    MissingConstraint(ConstraintId),
    
    #[error("Fragment {0} references non-existent constraint {1}")]
    InvalidConstraint(FragmentId, ConstraintId),
    
    #[error("Circular dependency detected: {0:?}")]
    CircularDependency(Vec<FragmentId>),
    
    #[error("Boundary wire inconsistency: {0}")]
    BoundaryInconsistency(String),
    
    #[error("Fragment {0} has no constraints")]
    EmptyFragment(FragmentId),
}

/// Validation result
#[derive(Debug)]
pub struct FragmentValidationResult {
    pub is_valid: bool,
    pub errors: Vec<FragmentValidationError>,
    pub warnings: Vec<String>,
}

impl FragmentValidationResult {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    pub fn add_error(&mut self, error: FragmentValidationError) {
        self.is_valid = false;
        self.errors.push(error);
    }
    
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

/// Validate a fragmentation result against the original graph
pub fn validate_fragmentation(
    result: &FragmentationResult,
    graph: &ConstraintGraph,
) -> FragmentValidationResult {
    let mut validation = FragmentValidationResult::valid();
    
    // Check 1: All constraints covered exactly once
    validate_constraint_coverage(result, graph, &mut validation);
    
    // Check 2: No empty fragments
    validate_no_empty_fragments(result, &mut validation);
    
    // Check 3: All referenced constraints exist
    validate_constraint_existence(result, graph, &mut validation);
    
    // Check 4: No circular dependencies
    validate_no_circular_dependencies(result, &mut validation);
    
    // Check 5: Boundary consistency
    validate_boundary_consistency(result, &mut validation);
    
    validation
}

fn validate_constraint_coverage(
    result: &FragmentationResult,
    graph: &ConstraintGraph,
    validation: &mut FragmentValidationResult,
) {
    let mut covered: HashSet<ConstraintId> = HashSet::new();
    
    for fragment in &result.fragments {
        for &constraint in &fragment.constraints {
            if covered.contains(&constraint) {
                validation.add_error(FragmentValidationError::DuplicateConstraint(constraint));
            } else {
                covered.insert(constraint);
            }
        }
    }
    
    // Check all graph constraints are covered
    for &constraint_id in graph.constraints.keys() {
        if !covered.contains(&constraint_id) {
            validation.add_error(FragmentValidationError::MissingConstraint(constraint_id));
        }
    }
}

fn validate_no_empty_fragments(
    result: &FragmentationResult,
    validation: &mut FragmentValidationResult,
) {
    for fragment in &result.fragments {
        if fragment.constraints.is_empty() {
            validation.add_error(FragmentValidationError::EmptyFragment(fragment.id));
        }
    }
}

fn validate_constraint_existence(
    result: &FragmentationResult,
    graph: &ConstraintGraph,
    validation: &mut FragmentValidationResult,
) {
    for fragment in &result.fragments {
        for &constraint in &fragment.constraints {
            if !graph.constraints.contains_key(&constraint) {
                validation.add_error(FragmentValidationError::InvalidConstraint(
                    fragment.id,
                    constraint,
                ));
            }
        }
    }
}

fn validate_no_circular_dependencies(
    result: &FragmentationResult,
    validation: &mut FragmentValidationResult,
) {
    // DFS-based cycle detection
    let mut visited: HashSet<FragmentId> = HashSet::new();
    let mut rec_stack: HashSet<FragmentId> = HashSet::new();
    let mut path: Vec<FragmentId> = Vec::new();
    
    for fragment in &result.fragments {
        if !visited.contains(&fragment.id) {
            if let Some(cycle) = detect_cycle(fragment.id, result, &mut visited, &mut rec_stack, &mut path) {
                validation.add_error(FragmentValidationError::CircularDependency(cycle));
            }
        }
    }
}

fn detect_cycle(
    id: FragmentId,
    result: &FragmentationResult,
    visited: &mut HashSet<FragmentId>,
    rec_stack: &mut HashSet<FragmentId>,
    path: &mut Vec<FragmentId>,
) -> Option<Vec<FragmentId>> {
    visited.insert(id);
    rec_stack.insert(id);
    path.push(id);
    
    if let Some(fragment) = result.get_fragment(id) {
        for &dep in &fragment.dependencies {
            if !visited.contains(&dep) {
                if let Some(cycle) = detect_cycle(dep, result, visited, rec_stack, path) {
                    return Some(cycle);
                }
            } else if rec_stack.contains(&dep) {
                // Found cycle
                let cycle_start = path.iter().position(|&x| x == dep).unwrap();
                return Some(path[cycle_start..].to_vec());
            }
        }
    }
    
    path.pop();
    rec_stack.remove(&id);
    None
}

fn validate_boundary_consistency(
    result: &FragmentationResult,
    validation: &mut FragmentValidationResult,
) {
    // For each output boundary, there should be a matching input boundary
    for fragment in &result.fragments {
        for boundary in &fragment.output_boundaries {
            // Find the target fragment
            let target = result.get_fragment(boundary.target_fragment);
            if target.is_none() {
                validation.add_error(FragmentValidationError::BoundaryInconsistency(
                    format!("Output boundary references non-existent fragment {}", boundary.target_fragment)
                ));
                continue;
            }
            
            let target = target.unwrap();
            
            // Check that target has matching input boundary
            let has_matching = target.input_boundaries.iter().any(|ib| {
                ib.wire_id == boundary.wire_id &&
                ib.source_fragment == fragment.id &&
                ib.target_fragment == boundary.target_fragment
            });
            
            if !has_matching {
                validation.add_error(FragmentValidationError::BoundaryInconsistency(
                    format!(
                        "Output boundary {} -> {} for wire {} has no matching input boundary",
                        fragment.id, boundary.target_fragment, boundary.wire_id
                    )
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fragment::BoundaryWire;
    use zk_fragment_graph::{WireId, Wire, Constraint, ConstraintType, ConstraintGraph};

    fn create_test_graph() -> ConstraintGraph {
        let mut graph = ConstraintGraph::new();
        
        for i in 0..3 {
            graph.add_wire(Wire::internal(WireId(i)));
        }
        
        let c0 = Constraint::new(ConstraintId(0), ConstraintType::Add, vec![], vec![WireId(0)]);
        let c1 = Constraint::new(ConstraintId(1), ConstraintType::Mul, vec![WireId(0)], vec![WireId(1)]);
        let c2 = Constraint::new(ConstraintId(2), ConstraintType::Add, vec![WireId(1)], vec![WireId(2)]);
        
        graph.add_constraint(c0);
        graph.add_constraint(c1);
        graph.add_constraint(c2);
        
        graph.build_edges_from_wires();
        graph
    }

    #[test]
    fn test_valid_fragmentation() {
        let graph = create_test_graph();
        
        let boundary = BoundaryWire {
            wire_id: WireId(0),
            source_fragment: FragmentId(0),
            target_fragment: FragmentId(1),
            source_index: 0,
            target_index: 0,
        };
        
        let mut f0 = FragmentSpec::new(FragmentId(0), vec![ConstraintId(0)]);
        f0.output_boundaries.push(boundary.clone());
        
        let mut f1 = FragmentSpec::new(FragmentId(1), vec![ConstraintId(1), ConstraintId(2)]);
        f1.input_boundaries.push(boundary);
        f1.dependencies.push(FragmentId(0));
        
        let result = FragmentationResult {
            fragments: vec![f0, f1],
            total_boundary_wires: 1,
            dependency_depth: 1,
            max_parallelism: 1,
            execution_order: vec![FragmentId(0), FragmentId(1)],
        };
        
        let validation = validate_fragmentation(&result, &graph);
        assert!(validation.is_valid, "Errors: {:?}", validation.errors);
    }

    #[test]
    fn test_duplicate_constraint_detection() {
        let graph = create_test_graph();
        
        let f0 = FragmentSpec::new(FragmentId(0), vec![ConstraintId(0), ConstraintId(1)]);
        let f1 = FragmentSpec::new(FragmentId(1), vec![ConstraintId(1), ConstraintId(2)]); // Duplicate!
        
        let result = FragmentationResult {
            fragments: vec![f0, f1],
            total_boundary_wires: 0,
            dependency_depth: 0,
            max_parallelism: 2,
            execution_order: vec![FragmentId(0), FragmentId(1)],
        };
        
        let validation = validate_fragmentation(&result, &graph);
        assert!(!validation.is_valid);
        assert!(validation.errors.iter().any(|e| matches!(e, FragmentValidationError::DuplicateConstraint(_))));
    }
}