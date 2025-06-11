use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum NodeStatus {
    Pending,
    Ready,
    Building,
    Complete,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct Node {
    pub target: String,
    pub dependencies: Vec<String>,
    pub commands: Vec<String>,
    pub output: Option<PathBuf>,
    pub state: NodeStatus,
}

impl Node {
    pub fn new(target: String) -> Node {
        Self {
            target,
            dependencies: Vec::new(),
            commands: Vec::new(),
            output: None,
            state: NodeStatus::Pending,
        }
    }
}
