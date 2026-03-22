use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "jao")]
#[command(about = "A tiny modern CLI example", long_about = None)]
struct Cli {}

fn main() {
    let _ = Cli::parse();
    println!("hello world");
}
