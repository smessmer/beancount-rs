use std::io::BufRead as _;

use beancount_rs::parse_account_component;
use chumsky::Parser as _;

fn main() {
    // read line from stdin
    let stdin = std::io::stdin();
    let handle = stdin.lock();
    println!("Parsing input:");
    for line in handle.lines() {
        let line = line.unwrap();
        // parse line as account component
        let account_component = parse_account_component().parse(&line);
        println!("{account_component:?}")
    }
}
