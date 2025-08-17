use chumsky::prelude::*;
use std::fmt::Write;

use crate::{
    model::Account,
    parser::chumsky::account::{
        account_component::marshal_account_component,
        account_type::{marshal_account_type, parse_account_type},
    },
};

pub fn parse_account<'a>() -> impl Parser<'a, &'a str, Account<'a>, extra::Err<Rich<'a, char>>> {
    parse_account_type()
        .then(
            just(':')
                .ignore_then(
                    crate::parser::chumsky::account::account_component::parse_account_component(),
                )
                .repeated()
                .collect(),
        )
        .map(|(account_type, components)| Account::new(account_type, components))
}

pub fn marshal_account(account: Account, writer: &mut impl Write) -> std::fmt::Result {
    marshal_account_type(account.account_type(), writer)?;
    for component in account.components() {
        write!(writer, ":")?;
        marshal_account_component(component, writer)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AccountComponent, AccountType};
    use chumsky::{
        error::{Rich, RichPattern},
        label::LabelError,
        util::Maybe,
    };
    use rstest::rstest;
    use rstest_reuse::*;
    use std::ops::Range;

    fn error(span: Range<usize>, message: &str) -> Rich<'_, char> {
        Rich::custom(SimpleSpan::from(span), message)
    }

    fn expected_found(
        span: Range<usize>,
        expected: impl IntoIterator<Item = impl Into<RichPattern<'static, char>>>,
        found: impl Into<Option<char>>,
    ) -> Rich<'static, char> {
        LabelError::<&str, _>::expected_found(
            expected,
            found.into().map(Maybe::Val),
            SimpleSpan::from(span),
        )
    }

    #[template]
    #[rstest]
    #[case("Assets:Cash", AccountType::Assets, vec!["Cash"])]
    #[case("Liabilities:Credit-Card", AccountType::Liabilities, vec!["Credit-Card"])]
    #[case("Income:Salary:Base", AccountType::Income, vec!["Salary", "Base"])]
    #[case("Expenses:Food:Restaurants:Fine-Dining", AccountType::Expenses, vec!["Food", "Restaurants", "Fine-Dining"])]
    #[case("Equity:Opening-Balances", AccountType::Equity, vec!["Opening-Balances"])]
    #[case("Assets:US:Bank:Checking", AccountType::Assets, vec!["US", "Bank", "Checking"])]
    #[case("Assets:401k:Company:Match", AccountType::Assets, vec!["401k", "Company", "Match"])]
    #[case("Expenses:Transport:Public-Transport", AccountType::Expenses, vec!["Transport", "Public-Transport"])]
    fn valid_account_template(
        #[case] input: &str,
        #[case] expected_type: AccountType,
        #[case] expected_components: Vec<&str>,
    ) {
    }

    #[template]
    #[rstest]
    #[case("assets:Cash", error(0..6, "Account component must start with an uppercase letter or a number"))] // Invalid account type (lowercase)
    #[case("InvalidType:Cash", error(0..11, "Expected Assets, Liabilities, Income, Expenses or Equity"))] // Invalid account type
    #[case("Assets:", error(7..7, "Account component cannot be empty"))] // Empty component
    #[case("Assets:cash", error(7..11, "Account component must start with an uppercase letter or a number"))] // Invalid component (lowercase start)
    #[case("Assets:Valid:", error(13..13, "Account component cannot be empty"))] // Empty component at end
    #[case("Assets::Valid", error(7..7, "Account component cannot be empty"))] // Empty component in middle
    #[case("Assets:Cash_Money", error(7..17, "Account component can only contain letters, numbers or dashes"))]
    #[case("Assets:Cash@Bank", error(7..16, "Account component can only contain letters, numbers or dashes"))]
    #[case("Assets:Cash Money", expected_found(11..12, [':'.into(), RichPattern::EndOfInput], ' '))] // Space in component
    #[case("Assets:-Invalid", error(7..15,"Account component must start with an uppercase letter or a number"))]
    #[case("", error(0..0, "Account component cannot be empty"))]
    #[case(":Cash", error(0..0, "Account component cannot be empty"))]
    #[case("Assets:Cash:", error(12..12, "Account component cannot be empty"))]
    fn invalid_account_template(#[case] input: &str, #[case] expected_error: Rich<char>) {}

    #[apply(valid_account_template)]
    fn parse_valid(
        #[case] input: &str,
        #[case] expected_type: AccountType,
        #[case] expected_components: Vec<&str>,
    ) {
        let result = parse_account().parse(input);
        assert!(
            result.has_output(),
            "Failed to parse valid account: {}",
            input
        );

        let parsed = result.into_result().unwrap();
        assert_eq!(parsed.account_type(), expected_type);

        let components: Vec<&str> = parsed.components().map(AsRef::as_ref).collect();
        assert_eq!(components, expected_components);
    }

    #[apply(invalid_account_template)]
    fn parse_invalid(#[case] input: &str, #[case] expected_error: Rich<char>) {
        let result = parse_account().parse(input);
        assert_eq!(vec![expected_error], result.into_errors(),);
    }

    #[apply(valid_account_template)]
    fn marshal(
        #[case] expected_output: &str,
        #[case] expected_type: AccountType,
        #[case] expected_components: Vec<&str>,
    ) {
        // Create account from expected components
        let components: Result<Vec<_>, _> = expected_components
            .into_iter()
            .map(AccountComponent::try_from)
            .collect();
        let components = components.unwrap();
        let account = Account::new(expected_type, components);

        // Marshal to string
        let mut output = String::new();
        let result = marshal_account(account, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, expected_output);
    }

    #[apply(valid_account_template)]
    fn marshal_and_parse(
        #[case] _input: &str,
        #[case] expected_type: AccountType,
        #[case] expected_components: Vec<&str>,
    ) {
        // Create original account
        let components: Result<Vec<_>, _> = expected_components
            .into_iter()
            .map(AccountComponent::try_from)
            .collect();
        let components = components.unwrap();
        let original_account = Account::new(expected_type, components);

        // Marshal to string
        let mut marshalled = String::new();
        marshal_account(original_account.clone(), &mut marshalled).unwrap();

        // Parse back from string
        let result = parse_account().parse(&marshalled);
        assert!(result.has_output());
        let parsed_account = result.into_result().unwrap();

        // Should be equal
        assert_eq!(
            original_account.account_type(),
            parsed_account.account_type()
        );
        let original_components: Vec<&str> =
            original_account.components().map(AsRef::as_ref).collect();
        let parsed_components: Vec<&str> = parsed_account.components().map(AsRef::as_ref).collect();
        assert_eq!(original_components, parsed_components);
    }

    #[test]
    fn parse_stops_at_whitespace() {
        // Test that parser correctly handles partial parsing
        let parser = parse_account().then_ignore(just(" ")).then(parse_account());
        let input = "Assets:Cash Liabilities:Credit-Card";
        let result = parser.parse(input);
        assert!(result.has_output());
        let (first, second) = result.into_result().unwrap();

        assert_eq!(first.account_type(), AccountType::Assets);
        let first_components: Vec<&str> = first.components().map(AsRef::as_ref).collect();
        assert_eq!(first_components, ["Cash"]);

        assert_eq!(second.account_type(), AccountType::Liabilities);
        let second_components: Vec<&str> = second.components().map(AsRef::as_ref).collect();
        assert_eq!(second_components, ["Credit-Card"]);
    }

    #[test]
    fn parse_single_component() {
        let result = parse_account().parse("Assets:Cash");
        assert!(result.has_output());
        let account = result.into_result().unwrap();
        assert_eq!(account.account_type(), AccountType::Assets);
        let components: Vec<&str> = account.components().map(AsRef::as_ref).collect();
        assert_eq!(components, ["Cash"]);
    }

    #[test]
    fn parse_multiple_components() {
        let result = parse_account().parse("Expenses:Food:Restaurants:Italian");
        assert!(result.has_output());
        let account = result.into_result().unwrap();
        assert_eq!(account.account_type(), AccountType::Expenses);
        let components: Vec<&str> = account.components().map(AsRef::as_ref).collect();
        assert_eq!(components, ["Food", "Restaurants", "Italian"]);
    }

    #[test]
    fn parse_numeric_components() {
        let result = parse_account().parse("Assets:401k:123Company");
        assert!(result.has_output());
        let account = result.into_result().unwrap();
        assert_eq!(account.account_type(), AccountType::Assets);
        let components: Vec<&str> = account.components().map(AsRef::as_ref).collect();
        assert_eq!(components, ["401k", "123Company"]);
    }

    #[test]
    fn marshal_empty_components() {
        // Test edge case with no components (should not be possible through parser but let's test marshal)
        let account = Account::new(AccountType::Assets, vec![]);
        let mut output = String::new();
        let result = marshal_account(account, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "Assets");
    }
}
