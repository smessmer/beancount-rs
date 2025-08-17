use std::io::BufRead as _;

use ariadne::Source;
use beancount_rs::{ParseResultExt as _, parse_account_component};
use chumsky::Parser as _;

fn main() {
    // read line from stdin
    let stdin = std::io::stdin();
    let handle = stdin.lock();
    println!("Input:");
    for line in handle.lines() {
        let line = line.unwrap();
        // parse line as account component
        let account_component = parse_account_component().parse(&line);
        for error_report in account_component.get_formatted_errors() {
            error_report.print(Source::from(&line)).unwrap();
        }
        if let Ok(account_component) = account_component.into_result() {
            println!("\nParsed: {:#?}\n\nInput:", account_component);
        }
    }
}
