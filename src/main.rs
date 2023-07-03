use clap::Parser;

use graph::{node::Node, Graph};
use std::{env, fs::File, rc::Rc};

mod graph;
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Filename
    #[arg(short, long)]
    file: String,

    /// Number of threads
    #[arg(short = 'j', long, default_value_t = 1)]
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

    let g = Graph::parse_file(f);
    g.detect_cycle();

    println!("Printing subgoals");
    for key in g.adj_list.borrow().keys() {
        println!("{}: {:?}", key.get_goal(), key.get_subgoals());
    }
    println!("Printing commands");
    for key in g.adj_list.borrow().keys() {
        println!("{}: {:?}", key.get_goal(), key.get_commands());
    }
    println!("Printing graph"); 
    for (key, value) in g.adj_list.borrow().iter() {
        println!("{}: {:?}", key.get_goal(), value.into_iter().map(|x| x.get_goal()).collect::<Vec<_>>()); 
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
