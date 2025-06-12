use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use tokio::process::Command;
use tokio::sync::{RwLock, Semaphore};
use tokio::time::{sleep, Duration};

use crate::graph::node::{Node, NodeStatus};

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

    pub async fn run_targets(&self, targets: Vec<String>) {
        println!("Running targets: {:?}", targets);

        // If no targets specified, use default target
        let targets_to_run = if targets.is_empty() {
            if self.default_target.is_empty() {
                println!("No targets specified and no default target found");
                return;
            }
            vec![self.default_target.clone()]
        } else {
            targets
        };

        // Get topological sort to determine build order
        let build_order = match self.topo_sort() {
            Ok(order) => order,
            Err(e) => {
                eprintln!("Cannot determine build order: {:?}", e);
                return;
            }
        };

        println!("Full build order: {:?}", build_order);

        // Filter build order to only include targets we need to build
        let required_targets = self.get_required_targets(&targets_to_run).await;
        let filtered_order: Vec<String> = build_order
            .into_iter()
            .filter(|target| required_targets.contains(target))
            .collect();

        println!("Filtered build order: {:?}", filtered_order);

        // Execute tasks using async task pool
        match self.execute_with_async_pool(filtered_order).await {
            Ok(()) => println!("All targets built successfully!"),
            Err(e) => eprintln!("Build failed: {}", e),
        }
    }

    async fn execute_with_async_pool(&self, build_order: Vec<String>) -> Result<(), String> {
        // Mark initially ready targets (no dependencies)
        {
            let mut nodes = self.nodes.write().await;
            for target in &build_order {
                if let Some(node) = nodes.get_mut(target) {
                    if node.dependencies.is_empty() {
                        node.state = NodeStatus::Ready;
                    }
                }
            }
        }

        // Spawn tasks for each target - they will wait until ready
        let mut task_handles = Vec::new();

        for target in build_order.clone() {
            let nodes_clone = Arc::clone(&self.nodes);
            let semaphore_clone = Arc::clone(&self.build_semaphore);
            let build_order_clone = build_order.clone();

            let handle = tokio::spawn(async move {
                // Wait until this target is ready
                loop {
                    let is_ready = {
                        let nodes = nodes_clone.read().await;
                        if let Some(node) = nodes.get(&target) {
                            match &node.state {
                                NodeStatus::Ready => true,
                                NodeStatus::Failed(_) | NodeStatus::Complete => return, // Don't build if failed/complete
                                NodeStatus::Pending => {
                                    // Check if dependencies are complete
                                    let all_deps_complete = node.dependencies.iter().all(|dep| {
                                        if let Some(dep_node) = nodes.get(dep) {
                                            matches!(dep_node.state, NodeStatus::Complete)
                                        } else {
                                            false
                                        }
                                    });
                                    all_deps_complete
                                }
                                NodeStatus::Building => false, // Wait if currently building
                            }
                        } else {
                            false
                        }
                    };

                    if is_ready {
                        // Mark as ready if dependencies are complete
                        let mut nodes = nodes_clone.write().await;
                        if let Some(node) = nodes.get_mut(&target) {
                            if matches!(node.state, NodeStatus::Pending) {
                                node.state = NodeStatus::Ready;
                            }
                        }
                        break;
                    }

                    // Wait a bit before checking again
                    sleep(Duration::from_millis(10)).await;
                }

                // Acquire semaphore permit for concurrent execution
                let _permit = semaphore_clone.acquire().await.unwrap();

                // Mark as building
                {
                    let mut nodes = nodes_clone.write().await;
                    if let Some(node) = nodes.get_mut(&target) {
                        node.state = NodeStatus::Building;
                    }
                }

                println!("Building target: {}", target);

                // Execute the target
                let result = Self::execute_target(&nodes_clone, &target).await;

                // Update status based on result
                {
                    let mut nodes = nodes_clone.write().await;
                    if let Some(node) = nodes.get_mut(&target) {
                        match result {
                            Ok(()) => {
                                node.state = NodeStatus::Complete;
                                println!("Completed target: {}", target);
                            }
                            Err(e) => {
                                node.state = NodeStatus::Failed(e.clone());
                                eprintln!("Failed target '{}': {}", target, e);
                            }
                        }
                    }
                }
            });

            task_handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in task_handles {
            handle
                .await
                .map_err(|e| format!("Task join error: {}", e))?;
        }

        // Check final status
        let nodes = self.nodes.read().await;
        let mut has_failures = false;

        println!("\nFinal Status:");
        for target in &build_order {
            if let Some(node) = nodes.get(target) {
                println!("  {}: {:?}", target, node.state);
                if matches!(node.state, NodeStatus::Failed(_)) {
                    has_failures = true;
                }
            }
        }

        if has_failures {
            Err("Build completed with failures".to_string())
        } else {
            Ok(())
        }
    }

    async fn execute_target(
        nodes: &Arc<RwLock<HashMap<String, Node>>>,
        target: &str,
    ) -> Result<(), String> {
        let nodes_guard = nodes.read().await;
        let node = nodes_guard
            .get(target)
            .ok_or_else(|| format!("Target '{}' not found", target))?;

        // Execute each command for this target
        for command in &node.commands {
            println!("  Executing: {}", command);

            let output = Command::new("sh")
                .arg("-c")
                .arg(command)
                .output()
                .await
                .map_err(|e| format!("Failed to execute command '{}': {}", command, e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                return Err(format!(
                    "Command '{}' failed with exit code {:?}\nstdout: {}\nstderr: {}",
                    command,
                    output.status.code(),
                    stdout,
                    stderr
                ));
            }

            // Print command output if any
            if !output.stdout.is_empty() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                print!("{}", stdout);
            }
        }

        Ok(())
    }

    async fn get_required_targets(&self, targets: &[String]) -> HashSet<String> {
        let nodes = self.nodes.read().await;
        let mut required = HashSet::new();
        let mut to_visit: Vec<String> = targets.to_vec();

        while let Some(current) = to_visit.pop() {
            if required.contains(&current) {
                continue; // Already processed
            }

            if let Some(node) = nodes.get(&current) {
                required.insert(current.clone());
                // Add dependencies to visit list
                for dep in &node.dependencies {
                    if !required.contains(dep) {
                        to_visit.push(dep.clone());
                    }
                }
            } else {
                eprintln!("Warning: Target '{}' not found", current);
            }
        }

        required
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
