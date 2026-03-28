use crate::types::{ConstraintGraph, ConstraintId, WireRole};
use std::io::Write;

/// Export graph to DOT format for visualization with Graphviz
pub fn to_dot(graph: &ConstraintGraph) -> String {
    let mut dot = String::new();
    
    dot.push_str("digraph ConstraintGraph {\n");
    dot.push_str("    rankdir=TB;\n");
    dot.push_str("    node [shape=box];\n\n");
    
    // Add nodes
    for (id, constraint) in &graph.constraints {
        let label = format!("{:?}", constraint.constraint_type);
        let color = match constraint.constraint_type {
            crate::types::ConstraintType::PublicInput => "lightgreen",
            crate::types::ConstraintType::PublicOutput => "lightblue",
            crate::types::ConstraintType::Mul => "lightyellow",
            crate::types::ConstraintType::Add => "lightgray",
            _ => "white",
        };
        
        dot.push_str(&format!(
            "    {} [label=\"{}\\n{}\" style=filled fillcolor=\"{}\"];\n",
            id.0, id, label, color
        ));
    }
    
    dot.push_str("\n");
    
    // Add edges
    for edge in &graph.edges {
        dot.push_str(&format!(
            "    {} -> {} [label=\"{}\"];\n",
            edge.from.0, edge.to.0, edge.wire
        ));
    }
    
    dot.push_str("}\n");
    
    dot
}

/// Export graph to DOT format with fragment coloring
pub fn to_dot_with_fragments(
    graph: &ConstraintGraph,
    fragment_assignment: &std::collections::HashMap<ConstraintId, usize>,
) -> String {
    let colors = [
        "lightblue", "lightgreen", "lightyellow", "lightpink",
        "lightcoral", "lightcyan", "lavender", "peachpuff",
    ];
    
    let mut dot = String::new();
    
    dot.push_str("digraph ConstraintGraph {\n");
    dot.push_str("    rankdir=TB;\n");
    dot.push_str("    node [shape=box];\n\n");
    
    // Add nodes with fragment coloring
    for (id, constraint) in &graph.constraints {
        let label = format!("{:?}", constraint.constraint_type);
        let fragment_id = fragment_assignment.get(id).copied().unwrap_or(0);
        let color = colors[fragment_id % colors.len()];
                dot.push_str(&format!(
            "    {} [label=\"{}\\n{}\\nF{}\" style=filled fillcolor=\"{}\"];\n",
            id.0, id, label, fragment_id, color
        ));
    }
    
    dot.push_str("\n");
    
    // Add edges with different styles for cross-fragment edges
    for edge in &graph.edges {
        let from_frag = fragment_assignment.get(&edge.from).copied().unwrap_or(0);
        let to_frag = fragment_assignment.get(&edge.to).copied().unwrap_or(0);
        
        let style = if from_frag != to_frag {
            "style=bold color=red"
        } else {
            ""
        };
        
        dot.push_str(&format!(
            "    {} -> {} [label=\"{}\" {}];\n",
            edge.from.0, edge.to.0, edge.wire, style
        ));
    }
    
    dot.push_str("}\n");
    
    dot
}

/// Export graph to JSON format
pub fn to_json(graph: &ConstraintGraph) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(graph)
}

/// Save graph to DOT file
pub fn save_dot<P: AsRef<std::path::Path>>(
    graph: &ConstraintGraph,
    path: P,
) -> std::io::Result<()> {
    let dot = to_dot(graph);
    let mut file = std::fs::File::create(path)?;
    file.write_all(dot.as_bytes())?;
    Ok(())
}

/// Generate ASCII art representation of the graph (simple version)
pub fn to_ascii(graph: &ConstraintGraph) -> String {
    let mut output = String::new();
    
    output.push_str("Constraint Graph\n");
    output.push_str("================\n\n");
    
    // Show constraints
    output.push_str("Constraints:\n");
    for (id, constraint) in &graph.constraints {
        output.push_str(&format!(
            "  {} [{:?}]: inputs={:?}, outputs={:?}\n",
            id, constraint.constraint_type, constraint.input_wires, constraint.output_wires
        ));
    }
    
    output.push_str("\nEdges (dependencies):\n");
    for edge in &graph.edges {
        output.push_str(&format!(
            "  {} -> {} (via {})\n",
            edge.from, edge.to, edge.wire
        ));
    }
    
    // Show wire summary
    output.push_str("\nWire Summary:\n");
    let public_inputs: Vec<_> = graph.wires.values()
        .filter(|w| matches!(w.role, WireRole::PublicInput))
        .map(|w| w.id)
        .collect();
    let public_outputs: Vec<_> = graph.wires.values()
        .filter(|w| matches!(w.role, WireRole::PublicOutput))
        .map(|w| w.id)
        .collect();
    
    output.push_str(&format!("  Public Inputs: {:?}\n", public_inputs));
    output.push_str(&format!("  Public Outputs: {:?}\n", public_outputs));
    output.push_str(&format!("  Total Wires: {}\n", graph.wires.len()));
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn create_test_graph() -> ConstraintGraph {
        let mut graph = ConstraintGraph::new();
        
        graph.add_wire(Wire::public_input(WireId(0)));
        graph.add_wire(Wire::internal(WireId(1)));
        graph.add_wire(Wire::public_output(WireId(2)));
        
        let c_a = Constraint::new(ConstraintId(0), ConstraintType::Add, vec![WireId(0)], vec![WireId(1)]);
        let c_b = Constraint::new(ConstraintId(1), ConstraintType::Mul, vec![WireId(1)], vec![WireId(2)]);
        
        graph.add_constraint(c_a);
        graph.add_constraint(c_b);
        
        graph.wires.get_mut(&WireId(1)).unwrap().producer = Some(ConstraintId(0));
        graph.wires.get_mut(&WireId(1)).unwrap().consumers.push(ConstraintId(1));
        graph.wires.get_mut(&WireId(2)).unwrap().producer = Some(ConstraintId(1));
        
        graph.build_edges_from_wires();
        graph
    }

    #[test]
    fn test_to_dot() {
        let graph = create_test_graph();
        let dot = to_dot(&graph);
        
        assert!(dot.contains("digraph"));
        assert!(dot.contains("->"));
    }

    #[test]
    fn test_to_ascii() {
        let graph = create_test_graph();
        let ascii = to_ascii(&graph);
        
        assert!(ascii.contains("Constraint Graph"));
        assert!(ascii.contains("Edges"));
    }
}