use chumsky::prelude::*;
use std::fmt::Write;

use crate::model::Commodity;

pub fn parse_commodity<'a>() -> impl Parser<'a, &'a str, Commodity<'a>, extra::Err<Rich<'a, char>>>
{
    any()
        .filter(|c: &char| !c.is_whitespace() && *c != ',')
        .repeated()
        .to_slice()
        .try_map(|slice: &'a str, span| {
            Commodity::try_from(slice)
                .map_err(|e| chumsky::error::Rich::custom(span, format!("{}", e)))
        })
}

pub fn marshal_commodity(commodity: Commodity, writer: &mut impl Write) -> std::fmt::Result {
    write!(writer, "{}", commodity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::InvalidCommodityError;
    use chumsky::span::SimpleSpan;
    use rstest::rstest;
    use rstest_reuse::*;

    #[template]
    #[rstest]
    #[case("USD")]
    #[case("EUR")]
    #[case("GBP")]
    #[case("BTC")]
    #[case("ETH")]
    #[case("SPY")]
    #[case("VTI")]
    #[case("AAPL")]
    #[case("GOOGL")]
    #[case("A")]
    #[case("A1")]
    #[case("AB1")]
    #[case("A'B")]
    #[case("A.B")]
    #[case("A_B")]
    #[case("A-B")]
    #[case("A'B.C_D-E1")]
    #[case("ABCDEFGHIJKLMNOPQR123456")] // Max length (24 chars)
    fn valid_commodity_template(#[case] input: &str) {}

    #[template]
    #[rstest]
    #[case("", InvalidCommodityError::Empty)]
    #[case("usd", InvalidCommodityError::InvalidStart)]
    #[case("1USD", InvalidCommodityError::InvalidStart)]
    #[case("-USD", InvalidCommodityError::InvalidStart)]
    #[case("_USD", InvalidCommodityError::InvalidStart)]
    #[case(".USD", InvalidCommodityError::InvalidStart)]
    #[case("'USD", InvalidCommodityError::InvalidStart)]
    #[case("USd", InvalidCommodityError::InvalidEnd)]
    #[case("USD-", InvalidCommodityError::InvalidEnd)]
    #[case("USD_", InvalidCommodityError::InvalidEnd)]
    #[case("USD.", InvalidCommodityError::InvalidEnd)]
    #[case("USD'", InvalidCommodityError::InvalidEnd)]
    #[case("US@D", InvalidCommodityError::InvalidCharacter)]
    #[case("US#D", InvalidCommodityError::InvalidCharacter)]
    #[case("US$D", InvalidCommodityError::InvalidCharacter)]
    #[case("US%D", InvalidCommodityError::InvalidCharacter)]
    #[case("US!D", InvalidCommodityError::InvalidCharacter)]
    #[case("USd1", InvalidCommodityError::InvalidCharacter)] // lowercase in middle
    #[case("ABCDEFGHIJKLMNOPQRSTUVWXY", InvalidCommodityError::TooLong)] // 25 characters
    fn invalid_commodity_template(input: &str, expected_error: InvalidCommodityError) {}

    #[apply(valid_commodity_template)]
    fn parse_valid(#[case] input: &str) {
        let result = parse_commodity().parse(input);
        assert!(result.has_output());
        let parsed = result.into_result().unwrap();
        assert_eq!(parsed.as_ref(), input);
    }

    #[apply(invalid_commodity_template)]
    fn parse_invalid(#[case] input: &str, #[case] expected_error: InvalidCommodityError) {
        let result = parse_commodity().parse(input);
        assert_eq!(
            vec![chumsky::error::Rich::custom(
                SimpleSpan::from(0..input.len()),
                expected_error.to_string(),
            )],
            result.into_errors(),
        );
    }

    #[apply(valid_commodity_template)]
    #[rstest]
    fn marshal(input: &str) {
        let commodity = Commodity::try_from(input).unwrap();
        let mut output = String::new();
        let result = marshal_commodity(commodity, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, input);
    }

    #[apply(valid_commodity_template)]
    #[rstest]
    fn marshal_and_parse(input: &str) {
        let original_commodity = Commodity::try_from(input).unwrap();

        // Marshal to string
        let mut marshalled = String::new();
        marshal_commodity(original_commodity.clone(), &mut marshalled).unwrap();

        // Parse back from string
        let result = parse_commodity().parse(&marshalled);
        assert!(result.has_output());
        let parsed_commodity = result.into_result().unwrap();

        // Should be equal
        assert_eq!(original_commodity, parsed_commodity);
    }

    #[test]
    fn parse_with_partial_input() {
        // Test that parser correctly handles partial parsing with different boundaries
        let parser = parse_commodity()
            .then_ignore(just(" "))
            .then(parse_commodity());
        let input = "USD EUR";
        let result = parser.parse(input);
        assert!(result.has_output());
        let (first, second) = result.into_result().unwrap();
        assert_eq!(first.as_ref(), "USD");
        assert_eq!(second.as_ref(), "EUR");
    }
}
