use clap::Parser; 

use graph::{parse_file, Graph, node::Node};
use std::{env, fs::File, rc::Rc};

mod graph;
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Filename
    #[arg(short, long)]
    file: String,

    /// Number of threads
    #[arg(short='j', long, default_value_t=1)]
    threads: u8,

    /// Targets to run
    targets: Vec<String>,
}

fn main() {
    let args = Args::parse(); 

    let f = match File::open(args.file) {
        Ok(f) => f, 
        Err(e) => panic!("{}", e),
    }; 

    let g = parse_file(f);

    println!("Printing subgoals"); 
    for key in g.nodes.keys() {
        println!("{}: {:?}", key.get_goal(), key.get_subgoals()); 
    }
}

fn gen_basic_graph() -> Graph {
    let mut g = Graph::new(); 

    let n1 = Rc::new(Node::new("a")); 
    let n2 = Rc::new(Node::new("b")); 
    let n3 = Rc::new(Node::new("c")); 
    let n4 = Rc::new(Node::new("d")); 

    g.insert(&n1); 
    g.insert(&n2); 
    g.insert(&n3); 
    g.insert(&n4); 

    g.add_neighbor(&n1, &n2); 
    g.add_neighbor(&n2, &n3); 
    g.add_neighbor(&n3, &n4); 
    g.add_neighbor(&n4, &n1); 

    g
}