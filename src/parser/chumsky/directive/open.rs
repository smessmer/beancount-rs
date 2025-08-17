use chumsky::{prelude::*, text::keyword};
use std::fmt::Write;

use crate::{
    model::DirectiveOpen,
    parser::chumsky::{
        account::{marshal_account, parse_account},
        commodity_list::{marshal_commodity_list, parse_commodity_list},
    },
};

/// Parser for open directive (without date)
/// Syntax: "open" <account> [<commodity_list>]
pub fn parse_open_directive<'a>()
-> impl Parser<'a, &'a str, DirectiveOpen<'a, 'a>, extra::Err<Rich<'a, char>>> {
    keyword("open")
        .ignore_then(parse_account().padded())
        .then(
            parse_commodity_list()
                .or_not()
                .map(|opt| opt.unwrap_or_default()),
        )
        .map(|(account, commodity_constraints)| DirectiveOpen::new(account, commodity_constraints))
}

/// Marshaller for open directive (without date)
pub fn marshal_open_directive(
    directive: &DirectiveOpen,
    writer: &mut impl Write,
) -> std::fmt::Result {
    write!(writer, "open ")?;

    marshal_account(directive.account().clone(), writer)?;

    // Marshal commodity constraints if any
    if directive.commodity_constraints().len() > 0 {
        write!(writer, " ")?;
        marshal_commodity_list(directive.commodity_constraints(), writer)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AccountType, Commodity, account};
    use common_macros::hash_set;
    use rstest::rstest;
    use rstest_reuse::*;
    use std::collections::HashSet;

    #[template]
    #[rstest]
    #[case("open Assets:Cash")]
    #[case("open Liabilities:CreditCard:CapitalOne")]
    #[case("open Assets:Checking USD")]
    #[case("open Assets:Investment USD, EUR")]
    #[case("open Assets:Crypto BTC,ETH,USDC")]
    #[case("open Expenses:Food")]
    #[case("open Income:Salary")]
    #[case("open Equity:Opening-Balances")]
    fn valid_open_directive_template(#[case] input: &str) {}

    #[apply(valid_open_directive_template)]
    fn parse_open_directive_valid(#[case] input: &str) {
        let result = parse_open_directive().parse(input);
        assert!(
            result.has_output(),
            "Failed to parse open directive: {}",
            input
        );
        let _parsed = result.into_result().unwrap();
    }

    #[apply(valid_open_directive_template)]
    fn marshal_and_parse_open_directive(#[case] input: &str) {
        // First parse the original
        let result = parse_open_directive().parse(input);
        assert!(result.has_output());
        let original = result.into_result().unwrap();

        // Marshal it
        let mut marshalled = String::new();
        let marshal_result = marshal_open_directive(&original, &mut marshalled);
        assert!(marshal_result.is_ok());

        // Parse it back
        let reparse_result = parse_open_directive().parse(&marshalled);
        assert!(reparse_result.has_output());
        let reparsed = reparse_result.into_result().unwrap();

        // Should be equal
        assert_eq!(original.account(), reparsed.account());
        assert_eq!(
            original.commodity_constraints().collect::<HashSet<_>>(),
            reparsed.commodity_constraints().collect::<HashSet<_>>()
        );
    }

    #[test]
    fn parse_open_directive_no_commodities() {
        let input = "open Assets:Cash";
        let result = parse_open_directive().parse(input);
        assert!(result.has_output());
        let directive = result.into_result().unwrap();

        assert_eq!(directive.account().account_type(), AccountType::Assets);
        let components: Vec<&str> = directive
            .account()
            .components()
            .map(AsRef::as_ref)
            .collect();
        assert_eq!(components, ["Cash"]);
        assert_eq!(directive.commodity_constraints().len(), 0);
    }

    #[test]
    fn parse_open_directive_single_commodity() {
        let input = "open Assets:Checking USD";
        let result = parse_open_directive().parse(input);
        assert!(result.has_output());
        let directive = result.into_result().unwrap();

        assert_eq!(directive.account().account_type(), AccountType::Assets);
        let components: Vec<&str> = directive
            .account()
            .components()
            .map(AsRef::as_ref)
            .collect();
        assert_eq!(components, ["Checking"]);
        assert_eq!(directive.commodity_constraints().len(), 1);
        assert!(
            directive
                .commodity_constraints()
                .any(|c| c.as_ref() == "USD")
        );
    }

    #[test]
    fn parse_open_directive_multiple_commodities_with_spaces() {
        let input = "open Assets:Investment USD, EUR, GBP";
        let result = parse_open_directive().parse(input);
        assert!(result.has_output());
        let directive = result.into_result().unwrap();

        assert_eq!(directive.account().account_type(), AccountType::Assets);
        let components: Vec<&str> = directive
            .account()
            .components()
            .map(AsRef::as_ref)
            .collect();
        assert_eq!(components, ["Investment"]);
        assert_eq!(directive.commodity_constraints().len(), 3);

        let commodity_strs: HashSet<&str> = directive
            .commodity_constraints()
            .map(|c| c.as_ref())
            .collect();
        assert_eq!(commodity_strs, hash_set!["USD", "EUR", "GBP"]);
    }

    #[test]
    fn parse_open_directive_multiple_commodities_without_spaces() {
        let input = "open Assets:Investment USD,EUR,GBP";
        let result = parse_open_directive().parse(input);
        assert!(result.has_output());
        let directive = result.into_result().unwrap();

        assert_eq!(directive.account().account_type(), AccountType::Assets);
        let components: Vec<&str> = directive
            .account()
            .components()
            .map(AsRef::as_ref)
            .collect();
        assert_eq!(components, ["Investment"]);
        assert_eq!(directive.commodity_constraints().len(), 3);

        let commodity_strs: HashSet<&str> = directive
            .commodity_constraints()
            .map(|c| c.as_ref())
            .collect();
        assert_eq!(commodity_strs, hash_set!["USD", "EUR", "GBP"]);
    }

    #[test]
    fn parse_open_directive_complex_account() {
        let input = "open Liabilities:CreditCard:CapitalOne USD";
        let result = parse_open_directive().parse(input);
        assert!(result.has_output());
        let directive = result.into_result().unwrap();

        assert_eq!(directive.account().account_type(), AccountType::Liabilities);
        let components: Vec<&str> = directive
            .account()
            .components()
            .map(AsRef::as_ref)
            .collect();
        assert_eq!(components, ["CreditCard", "CapitalOne"]);
        assert_eq!(directive.commodity_constraints().len(), 1);
        assert!(
            directive
                .commodity_constraints()
                .any(|c| c.as_ref() == "USD")
        );
    }

    #[test]
    fn marshal_open_directive_no_commodities() {
        let account = account!(Assets:Cash);
        let directive = DirectiveOpen::new(account, hash_set![]);

        let mut output = String::new();
        let result = marshal_open_directive(&directive, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "open Assets:Cash");
    }

    #[test]
    fn marshal_open_directive_with_commodities() {
        let account = account!(Assets:Investment);
        let commodities = hash_set![
            Commodity::try_from("USD").unwrap(),
            Commodity::try_from("EUR").unwrap(),
            Commodity::try_from("GBP").unwrap()
        ];
        let directive = DirectiveOpen::new(account, commodities);

        let mut output = String::new();
        let result = marshal_open_directive(&directive, &mut output);
        assert!(result.is_ok());
        // Commodities should be sorted alphabetically
        assert_eq!(output, "open Assets:Investment EUR,GBP,USD");
    }
}
