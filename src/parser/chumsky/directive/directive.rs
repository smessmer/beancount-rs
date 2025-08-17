use chumsky::{prelude::*, text::whitespace};
use std::fmt::Write;

use crate::{
    model::directive::{Directive, DirectiveContent},
    parser::chumsky::{
        date::parse_date,
        directive::open::{marshal_open_directive, parse_open_directive},
    },
};

/// Parser for complete directive with date
/// Syntax: <date> <directive_content>
pub fn parse_directive<'a>()
-> impl Parser<'a, &'a str, Directive<'a, 'a>, extra::Err<Rich<'a, char>>> {
    parse_date()
        .then_ignore(whitespace().at_least(1))
        .then(parse_directive_content())
        .map(|(date, content)| Directive::new(date, content))
}

fn parse_directive_content<'a>()
-> impl Parser<'a, &'a str, DirectiveContent<'a, 'a>, extra::Err<Rich<'a, char>>> {
    choice((
        parse_open_directive().map(DirectiveContent::Open),
        // TODO: Add more directive types here as they're implemented
    ))
}

pub fn marshal_directive(directive: &Directive, writer: &mut impl Write) -> std::fmt::Result {
    crate::parser::chumsky::date::marshal_date(directive.date(), writer)?;
    write!(writer, " ")?;

    // Marshal directive content
    marshal_directive_content(directive.content(), writer)
}

fn marshal_directive_content(
    content: &DirectiveContent,
    writer: &mut impl Write,
) -> std::fmt::Result {
    match content {
        DirectiveContent::Open(open) => marshal_open_directive(open, writer),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Commodity, account};
    use chrono::NaiveDate;
    use common_macros::hash_set;
    use rstest::rstest;
    use rstest_reuse::*;

    #[template]
    #[rstest]
    #[case("2024-01-01 open Assets:Cash")]
    #[case("2024-12-31 open Liabilities:CreditCard:CapitalOne")]
    #[case("2023-05-15 open Assets:Checking USD")]
    #[case("2024-03-20 open Assets:Investment USD,EUR")]
    #[case("2022-11-08 open Assets:Crypto BTC,ETH,USDC")]
    #[case("2024-01-01 open Expenses:Food")]
    #[case("2024-06-15 open Income:Salary")]
    #[case("2024-01-01 open Equity:Opening-Balances")]
    fn valid_directive_template(#[case] input: &str) {}

    #[apply(valid_directive_template)]
    fn parse_directive_valid(#[case] input: &str) {
        let result = parse_directive().parse(input);
        assert!(result.has_output(), "Failed to parse directive: {}", input);
        let _parsed = result.into_result().unwrap();
    }

    #[apply(valid_directive_template)]
    fn marshal_and_parse_directive(#[case] input: &str) {
        // First parse the original
        let result = parse_directive().parse(input);
        assert!(result.has_output());
        let original = result.into_result().unwrap();

        // Marshal it
        let mut marshalled = String::new();
        let marshal_result = marshal_directive(&original, &mut marshalled);
        assert!(marshal_result.is_ok());

        // Parse it back
        let reparse_result = parse_directive().parse(&marshalled);
        assert!(reparse_result.has_output());
        let reparsed = reparse_result.into_result().unwrap();

        // Should be equal
        assert_eq!(original, reparsed);
    }

    #[test]
    fn parse_directive_basic_open() {
        let input = "2024-01-01 open Assets:Cash";
        let result = parse_directive().parse(input);
        assert!(result.has_output());
        let directive = result.into_result().unwrap();

        assert_eq!(
            directive.date(),
            &NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
        );
        assert!(directive.as_open().is_some());

        let open = directive.as_open().unwrap();
        let components: Vec<&str> = open.account().components().map(AsRef::as_ref).collect();
        assert_eq!(components, ["Cash"]);
        assert_eq!(open.commodity_constraints().len(), 0);
    }

    #[test]
    fn parse_directive_open_with_commodities() {
        let input = "2024-03-15 open Assets:Investment USD,EUR,GBP";
        let result = parse_directive().parse(input);
        assert!(result.has_output());
        let directive = result.into_result().unwrap();

        assert_eq!(
            directive.date(),
            &NaiveDate::from_ymd_opt(2024, 3, 15).unwrap()
        );
        assert!(directive.as_open().is_some());

        let open = directive.as_open().unwrap();
        assert_eq!(open.commodity_constraints().len(), 3);

        let commodity_strs: std::collections::HashSet<&str> =
            open.commodity_constraints().map(|c| c.as_ref()).collect();
        assert_eq!(commodity_strs, hash_set!["USD", "EUR", "GBP"]);
    }

    #[test]
    fn parse_directive_complex_account() {
        let input = "2023-12-31 open Liabilities:CreditCard:CapitalOne:Rewards USD";
        let result = parse_directive().parse(input);
        assert!(result.has_output());
        let directive = result.into_result().unwrap();

        assert_eq!(
            directive.date(),
            &NaiveDate::from_ymd_opt(2023, 12, 31).unwrap()
        );
        assert!(directive.as_open().is_some());

        let open = directive.as_open().unwrap();
        let components: Vec<&str> = open.account().components().map(AsRef::as_ref).collect();
        assert_eq!(components, ["CreditCard", "CapitalOne", "Rewards"]);
        assert_eq!(open.commodity_constraints().len(), 1);
    }

    #[test]
    fn marshal_directive_basic_open() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let account = account!(Assets:Cash);
        let open_directive = crate::model::DirectiveOpen::new(account, hash_set![]);
        let directive = Directive::new_open(date, open_directive);

        let mut output = String::new();
        let result = marshal_directive(&directive, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "2024-01-01 open Assets:Cash");
    }

    #[test]
    fn marshal_directive_open_with_commodities() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let account = account!(Assets:Investment);
        let commodities = hash_set![
            Commodity::try_from("USD").unwrap(),
            Commodity::try_from("EUR").unwrap(),
            Commodity::try_from("GBP").unwrap()
        ];
        let open_directive = crate::model::DirectiveOpen::new(account, commodities);
        let directive = Directive::new_open(date, open_directive);

        let mut output = String::new();
        let result = marshal_directive(&directive, &mut output);
        assert!(result.is_ok());
        // Commodities should be sorted alphabetically
        assert_eq!(output, "2024-06-15 open Assets:Investment EUR,GBP,USD");
    }

    #[test]
    fn parse_directive_content_open() {
        let input = "open Assets:Cash USD";
        let result = parse_directive_content().parse(input);
        assert!(result.has_output());
        let content = result.into_result().unwrap();

        match content {
            DirectiveContent::Open(open) => {
                let components: Vec<&str> =
                    open.account().components().map(AsRef::as_ref).collect();
                assert_eq!(components, ["Cash"]);
                assert_eq!(open.commodity_constraints().len(), 1);
            }
        }
    }

    #[test]
    fn marshal_directive_content_open() {
        let account = account!(Assets:Checking);
        let commodities = hash_set![Commodity::try_from("USD").unwrap()];
        let open_directive = crate::model::DirectiveOpen::new(account, commodities);
        let content = DirectiveContent::Open(open_directive);

        let mut output = String::new();
        let result = marshal_directive_content(&content, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "open Assets:Checking USD");
    }

    #[test]
    fn parse_directive_multiple_spaces() {
        let input = "2024-01-01    open Assets:Cash";
        let result = parse_directive().parse(input);
        assert!(result.has_output());
        let directive = result.into_result().unwrap();

        assert_eq!(
            directive.date(),
            &NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
        );
        assert!(directive.as_open().is_some());
    }

    #[test]
    fn parse_directive_invalid_missing_space() {
        let input = "2024-01-01open Assets:Cash";
        let result = parse_directive().parse(input);
        assert!(!result.has_output());
    }

    #[test]
    fn parse_directive_invalid_date() {
        let input = "2024-13-01 open Assets:Cash";
        let result = parse_directive().parse(input);
        assert!(!result.has_output());
    }
}
