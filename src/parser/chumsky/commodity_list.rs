use chumsky::prelude::*;
use std::{collections::HashSet, fmt::Write};

use crate::{model::Commodity, parser::chumsky::commodity::parse_commodity};

pub fn parse_commodity_list<'a>()
-> impl Parser<'a, &'a str, HashSet<Commodity<'a>>, extra::Err<Rich<'a, char>>> {
    parse_commodity()
        .separated_by(just(',').padded())
        .at_least(1)
        .collect::<Vec<_>>()
        .map(|commodities| commodities.into_iter().collect())
}

pub fn marshal_commodity_list<'a, 'b>(
    commodities: impl Iterator<Item = &'a Commodity<'b>>,
    writer: &mut impl Write,
) -> std::fmt::Result
where
    'b: 'a,
{
    let mut sorted_commodities: Vec<_> = commodities.collect();
    sorted_commodities.sort_by(|a, b| a.cmp(b));
    let mut sorted_commodities = sorted_commodities.into_iter();

    if let Some(first) = sorted_commodities.next() {
        write!(writer, "{}", first)?;

        for commodity in sorted_commodities {
            write!(writer, ",{}", commodity)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Commodity;
    use common_macros::hash_set;
    use rstest::rstest;
    use rstest_reuse::*;

    #[template]
    #[rstest]
    #[case("USD", hash_set![Commodity::try_from("USD").unwrap()])]
    #[case("USD, EUR", hash_set![
        Commodity::try_from("USD").unwrap(),
        Commodity::try_from("EUR").unwrap()
    ])]
    #[case("USD,EUR,GBP", hash_set![
        Commodity::try_from("USD").unwrap(),
        Commodity::try_from("EUR").unwrap(),
        Commodity::try_from("GBP").unwrap()
    ])]
    #[case("BTC, ETH, USDC", hash_set![
        Commodity::try_from("BTC").unwrap(),
        Commodity::try_from("ETH").unwrap(),
        Commodity::try_from("USDC").unwrap()
    ])]
    fn valid_commodity_list_template(#[case] input: &str, #[case] expected: HashSet<Commodity>) {}

    #[apply(valid_commodity_list_template)]
    fn parse_commodity_list_valid(#[case] input: &str, #[case] expected: HashSet<Commodity>) {
        let result = parse_commodity_list().parse(input);
        assert!(
            result.has_output(),
            "Failed to parse commodity list: {}",
            input
        );
        let parsed = result.into_result().unwrap();
        assert_eq!(parsed, expected);
    }

    #[apply(valid_commodity_list_template)]
    fn marshal_commodity_list_test(#[case] _input: &str, #[case] commodities: HashSet<Commodity>) {
        let mut output = String::new();
        let result = marshal_commodity_list(commodities.iter(), &mut output);
        assert!(result.is_ok());

        // Parse the marshalled output back
        let parsed_result = parse_commodity_list().parse(&output);
        assert!(parsed_result.has_output());
        let parsed = parsed_result.into_result().unwrap();
        assert_eq!(parsed, commodities);
    }

    #[test]
    fn parse_commodity_list_single() {
        let result = parse_commodity_list().parse("USD");
        assert!(result.has_output());
        let parsed = result.into_result().unwrap();
        assert_eq!(parsed.len(), 1);
        assert!(parsed.contains(&Commodity::try_from("USD").unwrap()));
    }

    #[test]
    fn parse_commodity_list_with_spaces() {
        let result = parse_commodity_list().parse("USD , EUR , GBP");
        assert!(result.has_output());
        let parsed = result.into_result().unwrap();
        assert_eq!(parsed.len(), 3);
        assert!(parsed.contains(&Commodity::try_from("USD").unwrap()));
        assert!(parsed.contains(&Commodity::try_from("EUR").unwrap()));
        assert!(parsed.contains(&Commodity::try_from("GBP").unwrap()));
    }

    #[test]
    fn parse_commodity_list_no_spaces() {
        let result = parse_commodity_list().parse("USD,EUR,GBP");
        assert!(result.has_output());
        let parsed = result.into_result().unwrap();
        assert_eq!(parsed.len(), 3);
        assert!(parsed.contains(&Commodity::try_from("USD").unwrap()));
        assert!(parsed.contains(&Commodity::try_from("EUR").unwrap()));
        assert!(parsed.contains(&Commodity::try_from("GBP").unwrap()));
    }

    #[test]
    fn marshal_commodity_list_sorts_alphabetically() {
        let commodities = hash_set![
            Commodity::try_from("USD").unwrap(),
            Commodity::try_from("EUR").unwrap(),
            Commodity::try_from("GBP").unwrap(),
            Commodity::try_from("CAD").unwrap()
        ];

        let mut output = String::new();
        let result = marshal_commodity_list(commodities.iter(), &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "CAD,EUR,GBP,USD");
    }

    // Tests for invalid commodity lists
    #[rstest]
    #[case("", "Failed to parse empty commodity list")]
    #[case(",", "Failed to parse comma-only input")]
    #[case("USD,", "Failed to parse list ending with comma")]
    #[case(",USD", "Failed to parse list starting with comma")]
    #[case("USD,,EUR", "Failed to parse list with double comma")]
    #[case("USD, ,EUR", "Failed to parse list with space-only commodity")]
    #[case("usd", "Failed to parse lowercase commodity")]
    #[case("USD,usd", "Failed to parse list with invalid commodity")]
    #[case("1USD", "Failed to parse commodity starting with number")]
    #[case("USD,1EUR", "Failed to parse list with commodity starting with number")]
    #[case("US@D", "Failed to parse commodity with invalid character")]
    #[case(
        "USD,US@D",
        "Failed to parse list with commodity containing invalid character"
    )]
    #[case("USd", "Failed to parse commodity ending with lowercase")]
    #[case("USD,USd", "Failed to parse list with commodity ending with lowercase")]
    #[case("USD-", "Failed to parse commodity ending with dash")]
    #[case("USD,EUR-", "Failed to parse list with commodity ending with dash")]
    #[case("ABCDEFGHIJKLMNOPQRSTUVWXY", "Failed to parse too long commodity")]
    #[case(
        "USD,ABCDEFGHIJKLMNOPQRSTUVWXY",
        "Failed to parse list with too long commodity"
    )]
    fn parse_commodity_list_invalid(#[case] input: &str, #[case] description: &str) {
        let result = parse_commodity_list().parse(input);
        assert!(
            !result.has_output() || result.into_errors().len() > 0,
            "{}: {}",
            description,
            input
        );
    }
}
