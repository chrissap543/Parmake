use core::fmt;
use std::hash::Hash;

#[derive(Debug)]
pub struct Node {
    goal: String,
    subgoals: Vec<String>,
    commands: Vec<String>,
}

impl Node {
    pub fn new(goal: &str) -> Node {
        Node {
            goal: goal.to_string(),
            subgoals: vec![],
            commands: vec![],
        }
    }

    pub fn get_goal(&self) -> &String {
        &self.goal
    }

    pub fn get_subgoals(&self) -> &Vec<String> {
        &self.subgoals
    }

    pub fn set_subgoals(&mut self, subgoals: &Vec<String>) {
        self.subgoals = subgoals.clone();
    }

    pub fn push_command(&mut self, command: String) {
        self.commands.push(command);
    }

    pub fn get_commands(&self) -> &Vec<String> {
        &self.commands
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "goal: {}", self.goal)
    }
}

impl Hash for Node {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.goal.hash(state);
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.goal == other.goal
    }
}

impl Eq for Node {}
