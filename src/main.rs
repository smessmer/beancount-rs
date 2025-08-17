use std::io::BufRead as _;

use ariadne::Source;
use beancount_rs::{ParseResultExt as _, marshal_directive, parse_directive};
use chumsky::Parser as _;

fn main() {
    // read line from stdin
    let stdin = std::io::stdin();
    let handle = stdin.lock();
    println!("Input:");
    for line in handle.lines() {
        let line = line.unwrap();
        // parse line as account component
        let directive = parse_directive().parse(&line);
        for error_report in directive.get_formatted_errors() {
            error_report.print(Source::from(&line)).unwrap();
        }
        if let Ok(directive) = directive.into_result() {
            println!("\nParsed: {:#?}\n\nInput:", directive);
            let mut string = String::new();
            marshal_directive(&directive, &mut string).unwrap();
            println!("Marshalled: {}", string);
        }
    }
}
