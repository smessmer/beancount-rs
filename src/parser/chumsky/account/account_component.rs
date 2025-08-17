use chumsky::prelude::*;
use std::fmt::Write;

use crate::model::AccountComponent;

pub fn parse_account_component<'a>()
-> impl Parser<'a, &'a str, AccountComponent<'a>, extra::Err<Rich<'a, char>>> {
    any()
        .filter(|c: &char| !c.is_whitespace() && *c != ':')
        .repeated()
        .to_slice()
        .try_map(|slice: &'a str, span| {
            AccountComponent::try_from(slice)
                .map_err(|e| chumsky::error::Rich::custom(span, format!("{}", e)))
        })
}

pub fn marshal_account_component(
    component: &AccountComponent,
    writer: &mut impl Write,
) -> std::fmt::Result {
    write!(writer, "{}", component)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::InvalidAccountComponentError;
    use rstest::rstest;
    use rstest_reuse::*;

    #[template]
    #[rstest]
    #[case("Assets")]
    #[case("Liabilities")]
    #[case("Checking")]
    #[case("Credit-Card")]
    #[case("401k")]
    #[case("123Plan")]
    #[case("A")]
    #[case("A1")]
    #[case("Test-Component-123")]
    fn valid_component_template(#[case] input: &str) {}

    #[template]
    #[rstest]
    #[case("", InvalidAccountComponentError::Empty)]
    #[case("assets", InvalidAccountComponentError::InvalidStart)]
    #[case("checking", InvalidAccountComponentError::InvalidStart)]
    #[case("-Assets", InvalidAccountComponentError::InvalidStart)]
    #[case("_Assets", InvalidAccountComponentError::InvalidStart)]
    #[case("Assets_Checking", InvalidAccountComponentError::InvalidCharacter)]
    #[case("Assets@Bank", InvalidAccountComponentError::InvalidCharacter)]
    #[case("Assets Bank", InvalidAccountComponentError::InvalidCharacter)]
    #[case("assets-checking", InvalidAccountComponentError::InvalidCharacter)]
    #[case("Assets:Bank", InvalidAccountComponentError::InvalidCharacter)]
    fn invalid_component_template(input: &str, expected_error: InvalidAccountComponentError) {}

    #[apply(valid_component_template)]
    fn parse_valid(#[case] input: &str) {
        let result = parse_account_component().parse(input);
        assert!(result.has_output());
        let parsed = result.into_result().unwrap();
        assert_eq!(parsed.as_ref(), input);
    }

    #[apply(invalid_component_template)]
    fn parse_invalid(#[case] input: &str, #[case] expected_error: InvalidAccountComponentError) {
        let result = parse_account_component().parse(input);
        assert!(result.has_errors());
        // TODO
        // assert!(
        //     result.into_result().unwrap_err()[0]
        //         .to_string()
        //         .contains(&format!("{}", expected_error))
        // );
    }

    #[apply(valid_component_template)]
    #[rstest]
    fn marshal(input: &str) {
        let component = AccountComponent::try_from(input).unwrap();
        let mut output = String::new();
        let result = marshal_account_component(&component, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, input);
    }

    #[apply(valid_component_template)]
    #[rstest]
    fn marshal_and_parse(input: &str) {
        let original_component = AccountComponent::try_from(input).unwrap();

        // Marshal to string
        let mut marshalled = String::new();
        marshal_account_component(&original_component, &mut marshalled).unwrap();

        // Parse back from string
        let result = parse_account_component().parse(&marshalled);
        assert!(result.has_output());
        let parsed_component = result.into_result().unwrap();

        // Should be equal
        assert_eq!(original_component, parsed_component);
    }

    #[test]
    fn parse_with_partial_input() {
        // Test that parser correctly handles partial parsing
        let parser = parse_account_component()
            .then_ignore(just(":"))
            .then(parse_account_component());
        let input = "Assets:Checking";
        let result = parser.parse(input);
        assert!(result.has_output());
        let (first, second) = result.into_result().unwrap();
        assert_eq!(first.as_ref(), "Assets");
        assert_eq!(second.as_ref(), "Checking");
    }
}
