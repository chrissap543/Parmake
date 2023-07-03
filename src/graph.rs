use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::rc::Rc;

use self::node::Node;

pub mod node;

pub struct Graph {
    // remove pub after debugging
    pub adj_list: RefCell<HashMap<Rc<Node>, Vec<Rc<Node>>>>,
    pub nodes: HashMap<String, Rc<Node>>,
}

#[derive(PartialEq)]
enum NodeStatus {
    Inactive,
    Active,
    Visited,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            adj_list: RefCell::new(HashMap::new()),
            nodes: HashMap::new(),
        }
    }

    // returns if node already exist
    pub fn insert(&mut self, node: &Rc<Node>) -> bool {
        match self.adj_list.borrow_mut().entry(node.clone()) {
            Entry::Vacant(e) => {
                e.insert(vec![]);
                false
            }
            Entry::Occupied(_) => true,
        }
    }

    pub fn add_neighbor(&mut self, source: &Rc<Node>, dest: &Rc<Node>) {
        match self.adj_list.borrow_mut().entry(source.clone()) {
            Entry::Vacant(e) => {
                e.insert(vec![dest.clone()]);
            }
            Entry::Occupied(mut e) => {
                e.get_mut().push(dest.clone());
            }
        }
    }

    pub fn parse_file(f: File) -> Graph {
        let lines: Vec<String> = BufReader::new(f)
            .lines()
            .collect::<Vec<_>>()
            .into_iter()
            .map(|x| x.unwrap())
            .collect();

        let mut g = Graph::new();
        let mut target_names: Vec<String> = vec![];

        for mut idx in 0..lines.len() {
            if lines[idx].contains(":") {
                // target line
                let split: Vec<&str> = lines[idx].split(':').map(|x| x.trim()).collect();
                // split[0] is targets
                let mut targets: Vec<Node> = vec![];
                for target in split[0].split(" ") {
                    targets.push(Node::new(target));
                    target_names.push(target.to_string());
                }

                // split[1] is dependencies
                if split[1] != "" {
                    let deps: Vec<String> = split[1]
                        .split(' ')
                        .into_iter()
                        .map(|x| x.to_string())
                        .collect();
                    targets.iter_mut().for_each(|x| x.set_subgoals(&deps));
                }

                // parse commands
                let mut found_next = false;
                idx += 1;
                while !found_next && idx < lines.len() {
                    if lines[idx].contains(":") {
                        found_next = true;
                        idx -= 1;
                    } else {
                        targets
                            .iter_mut()
                            .for_each(|x| x.push_command(lines[idx].clone().trim().to_string()));
                        idx += 1;
                    }
                }

                // add to hashmaps
                for target in targets {
                    let tmp = Rc::new(target);
                    if let Some(_) = g.adj_list.borrow_mut().insert(tmp.clone(), vec![]) {
                        panic!("Improperly formatted makefile: Duplicate target");
                    }
                    if let Some(_) = g.nodes.insert(tmp.get_goal().to_string(), tmp.clone()) {
                        panic!("Improperly formatted makefile: Duplicate target");
                    }
                }
            } else {
                if idx == 0 {
                    panic!("Improperly formatted makefile: Command listed before target");
                }
            }
        }

        // add edges
        for target in target_names {
            for neighbor in g.nodes.get(&target).unwrap().get_subgoals() {
                g.adj_list
                    .borrow_mut()
                    .get_mut(g.nodes.get(&target).unwrap())
                    .unwrap()
                    .push(g.nodes.get(neighbor).unwrap().clone());
            }
        }

        g
    }

    // todo https://stackoverflow.com/a/38309110
    pub fn detect_cycle(&self) -> bool {
        let mut visited: HashMap<Rc<Node>, NodeStatus> = HashMap::new();

        for (key, value) in self.adj_list.borrow().iter() {
            visited.insert(key.clone(), NodeStatus::Inactive);
        }

        for (key, _) in self.adj_list.borrow().iter() {
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

            for neighbor in self.adj_list.borrow().get(&u.clone()).unwrap() {
                if self.visit(neighbor.clone(), visited) {
                    return true;
                }
            }

            visited.insert(u.clone(), NodeStatus::Visited);
        }

        false
    }
}
