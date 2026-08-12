mod cli;
mod config;
mod constants;
mod domain;
mod id;
mod output;
mod scanner;
mod store;
mod tasks;
mod tui;

fn main() {
    if let Err(err) = cli::run(std::env::args().collect()) {
        eprintln!("todolog: {err}");
        std::process::exit(1);
    }
}
