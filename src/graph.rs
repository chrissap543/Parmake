use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::rc::Rc;

use self::node::Node;

pub mod node;

pub struct Graph {
    // remove pub after debugging
    pub nodes: HashMap<Rc<Node>, Vec<Rc<Node>>>,
}

#[derive(PartialEq)]
enum NodeStatus {
    Inactive, 
    Active, 
    Visited,
}

pub fn parse_file(f: File) -> Graph {
    let lines: Vec<String> = BufReader::new(f)
        .lines()
        .collect::<Vec<_>>()
        .into_iter()
        .map(|x| x.unwrap())
        .collect();

    let mut g = Graph {
        nodes: HashMap::new(),
    }; 

    for mut idx in 0..lines.len() {
        if lines[idx].contains(":") {
            // target line
            let split: Vec<&str> = lines[idx].split(':').map(|x| x.trim()).collect(); 
            // split[0] is targets
            let mut targets: Vec<Node> = vec![]; 
            for target in split[0].split(" ") {
                targets.push(Node::new(target)); 
            }

            // split[1] is dependencies
            if split[1] != "" {
                let deps: Vec<String> = split[1].split(' ').into_iter().map(|x| x.to_string()).collect();
                targets.iter_mut().for_each(|x| x.set_subgoals(&deps)); 
            }

            for target in targets {
                if let Some(_) = g.nodes.insert(Rc::new(target), vec![]) {
                    panic!("Improperly formatted makefile: Multiple targets"); 
                }
            }
        } else {
            if idx == 0 {
                panic!("Improperly formatted makefile: Command listed before target"); 
            }
        }
    }

    g
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    // returns if node already exist
    pub fn insert(&mut self, node: &Rc<Node>) -> bool {
        match self.nodes.entry(node.clone()) {
            Entry::Vacant(e) => {
                e.insert(vec![]);
                false
            }
            Entry::Occupied(_) => true,
        }
    }

    pub fn add_neighbor(&mut self, source: &Rc<Node>, dest: &Rc<Node>) {
        match self.nodes.entry(source.clone()) {
            Entry::Vacant(e) => {
                e.insert(vec![dest.clone()]);
            }
            Entry::Occupied(mut e) => {
                e.get_mut().push(dest.clone());
            }
        }
    }

    // todo https://stackoverflow.com/a/38309110
    pub fn detect_cycle(&self) -> bool {
        let mut visited: HashMap<Rc<Node>, NodeStatus> = HashMap::new();

        for (key, value) in &self.nodes {
            visited.insert(key.clone(), NodeStatus::Inactive);
        }

        for (key, _) in &self.nodes {
            if visited[key] == NodeStatus::Inactive {
                if self.visit(key.clone(), &mut visited) {
                    return true;
                }
            }
        }

        false
    }

    fn visit(&self, u: Rc<Node>, visited: &mut HashMap<Rc<Node>, NodeStatus>) -> bool {
        if *visited.get(&u).unwrap() == NodeStatus::Active {
            return true; 
        }

        if *visited.get(&u).unwrap() == NodeStatus::Inactive {
            visited.insert(u.clone(), NodeStatus::Active); 
            
            for neighbor in self.nodes.get(&u.clone()).unwrap() {
                if self.visit(neighbor.clone(), visited) {
                    return true; 
                }
            }

            visited.insert(u.clone(), NodeStatus::Visited); 
        }

        false
    }
}
