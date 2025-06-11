use parmake::graph::node::Node;
use parmake::graph::CycleError;
use parmake::graph::Graph;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_cycle() {
        let graph = crate::Graph::new(4);
        let mut node_a = Node::new("a".to_string());
        node_a.dependencies = vec!["b".to_string()];

        let mut node_b = Node::new("b".to_string());
        node_b.dependencies = vec!["c".to_string()];

        let node_c = Node::new("c".to_string());

        graph.add_node(node_a).unwrap();
        graph.add_node(node_b).unwrap();
        graph.add_node(node_c).unwrap();

        assert!(!graph.detect_cycles());
        graph.debug_print();

        let topo_order = graph.topo_sort().unwrap();
        assert_eq!(topo_order.len(), 3);
        println!("{:?}", topo_order);
        // c should come before b, b should come before a
        let c_pos = topo_order.iter().position(|x| x == "c").unwrap();
        let b_pos = topo_order.iter().position(|x| x == "b").unwrap();
        let a_pos = topo_order.iter().position(|x| x == "a").unwrap();

        assert!(c_pos < b_pos);
        assert!(b_pos < a_pos);
    }

    #[test]
    fn test_simple_cycle() {
        let graph = Graph::new(4);

        // a -> b -> a (cycle)
        let mut node_a = Node::new("a".to_string());
        node_a.dependencies = vec!["b".to_string()];

        let mut node_b = Node::new("b".to_string());
        node_b.dependencies = vec!["a".to_string()];

        graph.add_node(node_a).unwrap();
        graph.add_node(node_b).unwrap();

        let result = graph.topo_sort();
        assert!(result.is_err());

        if let Err(CycleError::CircularDependency { cycle }) = result {
            assert!(cycle.len() >= 2);
            assert!(cycle.contains(&"a".to_string()));
            assert!(cycle.contains(&"b".to_string()));
        }
    }
}
