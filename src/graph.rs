use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{RwLock, Semaphore};

use crate::graph::node::Node;

pub mod node;

pub struct Graph {
    /// All nodes in the graph, keyed by target name
    pub nodes: Arc<RwLock<HashMap<String, Node>>>,
    /// Semaphore to limit concurrent builds
    pub build_semaphore: Arc<Semaphore>,
    pub default_target: String,
}

#[derive(Debug, PartialEq)]
pub enum CycleError {
    /// A circular dependency was found
    CircularDependency {
        cycle: Vec<String>,
    },
    /// A target was referenced but not defined
    MissingTarget {
        target: String,
    },

    LockError, // if we hit this, something has gone horribly wrong
}

impl Graph {
    pub fn new(num_threads: usize) -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            build_semaphore: Arc::new(Semaphore::new(num_threads)),
            default_target: String::new(),
        }
    }

    pub fn detect_cycles(&self) -> bool {
        match self.kahns() {
            Ok(_) => false,
            Err(_) => true,
        }
    }

    pub fn topo_sort(&self) -> Result<Vec<String>, CycleError> {
        self.kahns()
    }

    fn kahns(&self) -> Result<Vec<String>, CycleError> {
        let nodes = self.nodes.try_read().map_err(|_| CycleError::LockError)?;

        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut queue = Vec::new();
        let mut result = Vec::new();

        for (target, node) in nodes.iter() {
            // verify dependencies
            for dep in &node.dependencies {
                if !nodes.contains_key(dep) {
                    return Err(CycleError::MissingTarget {
                        target: dep.clone(),
                    });
                }
            }
            in_degree.insert(target.clone(), node.dependencies.len());
        }

        for (target, &degree) in &in_degree {
            if degree == 0 {
                println!("Found degree 0 node: {}", target);
                queue.push(target.clone());
            }
        }

        while let Some(current) = queue.pop() {
            result.push(current.clone());

            for (target, node) in nodes.iter() {
                if node.dependencies.contains(&current) {
                    let degree = in_degree.get_mut(target).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push(target.clone());
                    }
                }
            }
        }

        // not all nodes processed
        if result.len() != nodes.len() {
            let remaining: Vec<String> = nodes
                .keys()
                .filter(|&target| !result.contains(target))
                .cloned()
                .collect();
            return Err(CycleError::CircularDependency { cycle: remaining });
        }

        Ok(result)
    }

    /// Add a node to the graph (for testing)
    pub fn add_node(&self, node: Node) -> Result<(), CycleError> {
        let mut nodes = self.nodes.try_write().map_err(|_| CycleError::LockError)?;
        nodes.insert(node.target.clone(), node);
        Ok(())
    }

    /// Get a node (for testing/debugging)
    pub fn get_node(&self, target: &str) -> Result<Option<Node>, CycleError> {
        let nodes = self.nodes.try_read().map_err(|_| CycleError::LockError)?;
        Ok(nodes.get(target).cloned())
    }
}
