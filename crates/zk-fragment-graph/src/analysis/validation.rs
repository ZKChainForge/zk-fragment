use crate::types::{ConstraintGraph, ConstraintId, WireId};
use crate::algorithms;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ValidationError {
    #[error("Graph contains cycle involving constraint {0}")]
    CycleDetected(ConstraintId),
    
    #[error("Wire {0} has no producer but is not an input")]
    OrphanWire(WireId),
    
    #[error("Constraint {0} references non-existent wire {1}")]
    MissingWire(ConstraintId, WireId),
    
    #[error("Wire {0} has multiple producers: {1:?}")]
    MultipleProducers(WireId, Vec<ConstraintId>),
    
    #[error("Graph is disconnected: {0} components found")]
    Disconnected(usize),
    
    #[error("Constraint {0} has no inputs or outputs")]
    IsolatedConstraint(ConstraintId),
}

/// Result of graph validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    pub fn with_error(error: ValidationError) -> Self {
        Self {
            is_valid: false,
            errors: vec![error],
            warnings: Vec::new(),
        }
    }
    
    pub fn add_error(&mut self, error: ValidationError) {
        self.is_valid = false;
        self.errors.push(error);
    }
    
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

/// Validate that the graph is a proper DAG
pub fn validate_dag(graph: &ConstraintGraph) -> Result<(), ValidationError> {
    match algorithms::topological_sort(graph) {
        Ok(_) => Ok(()),
        Err(algorithms::TopologicalSortError::CycleDetected(id)) => {
            Err(ValidationError::CycleDetected(id))
        }
        Err(algorithms::TopologicalSortError::EmptyGraph) => Ok(()),
    }
}

/// Validate wire consistency
pub fn validate_wires(graph: &ConstraintGraph) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    
    // Check that all constraint-referenced wires exist
    for (constraint_id, constraint) in &graph.constraints {
        for &wire_id in &constraint.input_wires {
            if !graph.wires.contains_key(&wire_id) {
                errors.push(ValidationError::MissingWire(*constraint_id, wire_id));
            }
        }
        for &wire_id in &constraint.output_wires {
            if !graph.wires.contains_key(&wire_id) {
                errors.push(ValidationError::MissingWire(*constraint_id, wire_id));
            }
        }
    }
    
    // Check for multiple producers
    for (wire_id, _wire) in &graph.wires {
        let mut producers: Vec<ConstraintId> = Vec::new();
        
        for (constraint_id, constraint) in &graph.constraints {
            if constraint.output_wires.contains(wire_id) {
                producers.push(*constraint_id);
            }
        }
        
        if producers.len() > 1 {
            errors.push(ValidationError::MultipleProducers(*wire_id, producers));
        }
    }
    
    errors
}

/// Full validation of the constraint graph
pub fn validate(graph: &ConstraintGraph) -> ValidationResult {
    let mut result = ValidationResult::valid();
    
    // Check DAG property
    if let Err(e) = validate_dag(graph) {
        result.add_error(e);
    }
    
    // Check wire consistency
    for error in validate_wires(graph) {
        result.add_error(error);
    }
    
    // Check for isolated constraints (warning, not error)
    for (id, constraint) in &graph.constraints {
        if constraint.input_wires.is_empty() && constraint.output_wires.is_empty() {
            result.add_warning(format!("Constraint {} has no inputs or outputs", id));
        }
    }
    
    // Check connectivity (warning if disconnected)
    let scc = algorithms::find_sccs(graph);
    if scc.components.len() > 1 {
        result.add_warning(format!(
            "Graph has {} disconnected components",
            scc.components.len()
        ));
    }
    
    result
}

/// Check if the graph is suitable for fragmentation
pub fn check_fragmentation_suitability(graph: &ConstraintGraph) -> FragmentationSuitability {
    let mut suitability = FragmentationSuitability {
        is_suitable: true,
        min_recommended_fragments: 1,
        max_recommended_fragments: 1,
        reasons: Vec::new(),
    };
    
    let constraint_count = graph.constraints.len();
    
    // Too small to fragment
    if constraint_count < 10 {
        suitability.is_suitable = false;
        suitability.reasons.push("Circuit too small to benefit from fragmentation".into());
        return suitability;
    }
    
    // Check for cut vertices
    let cut_analysis = algorithms::find_cut_vertices(graph);
    if cut_analysis.cut_vertices.is_empty() {
        suitability.reasons.push("No natural cut points found; partitioning will create many boundaries".into());
    } else {
        suitability.reasons.push(format!(
            "Found {} natural cut points",
            cut_analysis.cut_vertices.len()
        ));
    }
    
    // Recommend fragment count
    suitability.min_recommended_fragments = 2;
    suitability.max_recommended_fragments = (constraint_count / 100).max(2).min(16);
    
    // Ideal is having cut vertices for fragmentation
    if cut_analysis.cut_vertices.len() >= 2 {
        suitability.min_recommended_fragments = cut_analysis.cut_vertices.len().min(4);
    }
    
    suitability
}

#[derive(Debug, Clone)]
pub struct FragmentationSuitability {
    pub is_suitable: bool,
    pub min_recommended_fragments: usize,
    pub max_recommended_fragments: usize,
    pub reasons: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn create_valid_graph() -> ConstraintGraph {
        let mut graph = ConstraintGraph::new();
        
        graph.add_wire(Wire::public_input(WireId(0)));
        graph.add_wire(Wire::internal(WireId(1)));
        
        let c_a = Constraint::new(ConstraintId(0), ConstraintType::Add, vec![WireId(0)], vec![WireId(1)]);
        graph.add_constraint(c_a);
        
        graph.wires.get_mut(&WireId(1)).unwrap().producer = Some(ConstraintId(0));
        graph.build_edges_from_wires();
        
        graph
    }

    #[test]
    fn test_validate_valid_graph() {
        let graph = create_valid_graph();
        let result = validate(&graph);
        
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_dag() {
        let graph = create_valid_graph();
        assert!(validate_dag(&graph).is_ok());
    }
}