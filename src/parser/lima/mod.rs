use std::path::Path;

use beancount_parser_lima::{BeancountParser, BeancountSources, ParseError, ParseSuccess};

use crate::{model::Directive, parser::lima::error::LimaConversionError};

mod account;
mod amount;
mod amount_with_tolerance;
mod commodity;
mod directive;
mod error;

pub fn parse(path: &Path) -> Result<(), ParseError> {
    let sources = BeancountSources::try_from(path).unwrap();
    let parser = BeancountParser::new(&sources);
    let parsed = parser.parse()?;
    let directives = ingest(&parsed).unwrap(); // TODO No unwrap
    println!("Directives: {:?}", directives);
    Ok(())
}

pub fn ingest<'a, 'r>(
    parsed: &'r ParseSuccess<'a>,
) -> Result<Vec<Directive<'a>>, LimaConversionError<'a>>
where
    'r: 'a,
{
    let directives = parsed
        .directives
        .iter()
        .map(|d| d.item().try_into())
        .collect::<Result<Vec<Directive>, LimaConversionError>>()?;
    Ok(directives)
}
