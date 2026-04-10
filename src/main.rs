use std::{env, process};

fn main() {
    let mut args = env::args();
    let name = args.next().unwrap_or_default();

    let path = args.next().unwrap_or_else(|| {
        eprintln!("usage: {name} <file>");
        process::exit(1);
    });

    toy_exchange::run(&path).unwrap_or_else(|e| {
        eprintln!("error: {e}");
        process::exit(1);
    });
}
