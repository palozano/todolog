mod cli;
mod config;
mod constants;
mod domain;
mod id;
mod scanner;
mod store;

fn main() {
    if let Err(err) = cli::run(std::env::args().collect()) {
        eprintln!("todolog: {err}");
        std::process::exit(1);
    }
}
