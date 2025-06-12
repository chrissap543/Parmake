use clap::Parser;
use graph::CycleError;
use graph::Graph;
use parser::Args;

mod graph;
mod parser;

#[tokio::main]
async fn main() -> Result<(), CycleError> {
    // let mut g = Graph::new(4);
    // if let Err(e) = g.parse_makefile("makefiles/Makefile") {
    //     eprintln!("{}", e.to_string());
    // }
    // g.debug_print();

    let args: Args = Parser::parse();
    let mut g = Graph::new(4);
    if let Err(e) = g.parse_makefile(&args.file) {
        eprintln!("{}", e.to_string());
        panic!();
    }
    g.debug_print();

    g.run_targets(args.targets).await;

    Ok(())
}
