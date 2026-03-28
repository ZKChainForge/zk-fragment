use zk_fragment_graph::prelude::*;

fn main() {
    println!("=== ZK-FRAGMENT Week 1: Graph Analysis ===\n");

    println!("Example 1: Simple Arithmetic Circuit");
    println!("=====================================");
    example_simple_circuit();

    println!("\n");

    println!("Example 2: Chain Circuit");
    println!("========================");
    example_chain_circuit();

    println!("\n");

    println!("Example 3: Binary Tree Circuit");
    println!("==============================");
    example_tree_circuit();
}

fn example_simple_circuit() {
    let mut builder = GraphBuilder::new();

    let x = builder.add_public_input();
    let y = builder.add_public_input();
    let sum = builder.add_addition(x, y);
    let product = builder.add_multiplication(sum, x);
    builder.mark_public_output(product);

    let graph = builder.build();
    print_graph_analysis(&graph);
}

fn example_chain_circuit() {
    let graph = create_chain_circuit(10);
    print_graph_analysis(&graph);
}

fn example_tree_circuit() {
    let graph = create_tree_circuit(4);
    print_graph_analysis(&graph);
}

fn print_graph_analysis(graph: &ConstraintGraph) {
    let stats = graph.compute_stats();
    println!("Basic Statistics:");
    println!("  Constraints: {}", stats.constraint_count);
    println!("  Wires: {}", stats.wire_count);
    println!("  Edges: {}", stats.edge_count);
    println!("  Density: {:.4}", stats.density);
    println!("  Longest Path: {}", stats.longest_path);

    println!(
        "\nDAG Validation: {}",
        if graph.is_dag() { "VALID" } else { "INVALID" }
    );

    match topological_sort(graph) {
        Ok(order) => {
            println!("\nTopological Order:");
            for (i, id) in order.iter().enumerate() {
                let constraint = &graph.constraints[id];
                println!("  {}. {} [{:?}]", i + 1, id, constraint.constraint_type);
            }
        }
        Err(e) => println!("\nTopological Sort Failed: {}", e),
    }

    let cut_analysis = find_cut_vertices(graph);
    println!("\nCut Vertices:");
    if cut_analysis.cut_vertices.is_empty() {
        println!("  None found");
    } else {
        for cv in &cut_analysis.cut_vertices {
            println!("  {}", cv);
        }
    }

    let detailed = compute_detailed_metrics(graph);
    println!(
        "\nFragmentation Score: {:.1}/100",
        detailed.fragmentation_score
    );

    println!("\n{}", to_ascii(graph));
}