use chumsky::{prelude::*, text::keyword};
use std::fmt::Write;

use crate::{
    model::DirectiveOpen,
    parser::chumsky::{
        account::{marshal_account, parse_account},
        commodity_list::{marshal_commodity_list, parse_commodity_list},
    },
};

const KEYWORD_OPEN: &str = "open";

/// Parser for open directive (without date)
/// Syntax: "open" <account> [<commodity_list>]
pub fn parse_open_directive<'a>()
-> impl Parser<'a, &'a str, DirectiveOpen<'a>, extra::Err<Rich<'a, char>>> {
    keyword(KEYWORD_OPEN)
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
    write!(writer, "{KEYWORD_OPEN} ")?;

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
    #[case("open Assets:Cash", AccountType::Assets, vec!["Cash"], vec![])]
    #[case("open Liabilities:CreditCard:CapitalOne", AccountType::Liabilities, vec!["CreditCard", "CapitalOne"], vec![])]
    #[case("open Assets:Checking USD", AccountType::Assets, vec!["Checking"], vec!["USD"])]
    #[case("open Assets:Investment USD, EUR", AccountType::Assets, vec!["Investment"], vec!["EUR", "USD"])]
    #[case("open Assets:Crypto BTC,ETH,USDC", AccountType::Assets, vec!["Crypto"], vec!["BTC", "ETH", "USDC"])]
    #[case("open Expenses:Food", AccountType::Expenses, vec!["Food"], vec![])]
    #[case("open Income:Salary", AccountType::Income, vec!["Salary"], vec![])]
    #[case("open Equity:Opening-Balances", AccountType::Equity, vec!["Opening-Balances"], vec![])]
    fn valid_open_directive_template(
        #[case] input: &str,
        #[case] expected_account_type: AccountType,
        #[case] expected_account_components: Vec<&str>,
        #[case] expected_commodities: Vec<&str>,
    ) {
    }

    #[apply(valid_open_directive_template)]
    fn parse_open_directive_valid(
        #[case] input: &str,
        #[case] expected_account_type: AccountType,
        #[case] expected_account_components: Vec<&str>,
        #[case] expected_commodities: Vec<&str>,
    ) {
        let result = parse_open_directive().parse(input);
        assert!(
            result.has_output(),
            "Failed to parse open directive: {}",
            input
        );
        let parsed = result.into_result().unwrap();

        // Validate account
        assert_eq!(parsed.account().account_type(), expected_account_type);
        let components: Vec<&str> = parsed.account().components().map(AsRef::as_ref).collect();
        assert_eq!(components, expected_account_components);

        // Validate commodities (sorted for consistent comparison)
        let mut actual_commodities: Vec<&str> =
            parsed.commodity_constraints().map(|c| c.as_ref()).collect();
        let mut expected_commodities_sorted = expected_commodities.clone();
        actual_commodities.sort();
        expected_commodities_sorted.sort();
        assert_eq!(actual_commodities, expected_commodities_sorted);
    }

    #[apply(valid_open_directive_template)]
    fn marshal_and_parse_open_directive(
        #[case] input: &str,
        #[case] _expected_account_type: AccountType,
        #[case] _expected_account_components: Vec<&str>,
        #[case] _expected_commodities: Vec<&str>,
    ) {
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
