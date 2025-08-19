use chumsky::{prelude::*, text::whitespace};
use std::fmt::Write;

use crate::{
    model::{Amount, directive::PostingAmount},
    parser::chumsky::amount::{marshal_amount, parse_amount},
};

/// Parser for posting amount with optional cost and price
/// Syntax: <amount> [{<cost>}] [@ <price>]
pub fn parse_posting_amount<'a>()
-> impl Parser<'a, &'a str, PostingAmount<'a>, extra::Err<Rich<'a, char>>> {
    parse_amount()
        .then(parse_cost().or_not())
        .then(parse_price().or_not())
        .map(|((amount, cost), price)| {
            let mut posting_amount = PostingAmount::new(amount);
            if let Some(cost) = cost {
                posting_amount = posting_amount.with_cost(cost);
            }
            if let Some(price) = price {
                posting_amount = posting_amount.with_price(price);
            }
            posting_amount
        })
}

fn parse_cost<'a>() -> impl Parser<'a, &'a str, Amount<'a>, extra::Err<Rich<'a, char>>> {
    whitespace()
        .at_least(1)
        .ignore_then(just('{'))
        .ignore_then(whitespace())
        .ignore_then(parse_amount())
        .then_ignore(whitespace())
        .then_ignore(just('}'))
}

fn parse_price<'a>() -> impl Parser<'a, &'a str, Amount<'a>, extra::Err<Rich<'a, char>>> {
    whitespace()
        .at_least(1)
        .ignore_then(just('@'))
        .ignore_then(whitespace().at_least(1))
        .ignore_then(parse_amount())
}

pub fn marshal_posting_amount(
    posting_amount: &PostingAmount,
    writer: &mut impl Write,
) -> std::fmt::Result {
    // Write the amount
    marshal_amount(posting_amount.amount(), writer)?;

    // Write cost if present
    if let Some(cost) = posting_amount.cost() {
        write!(writer, " {{")?;
        marshal_amount(cost, writer)?;
        write!(writer, "}}")?;
    }

    // Write price if present
    if let Some(price) = posting_amount.price() {
        write!(writer, " @ ")?;
        marshal_amount(price, writer)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::commodity;
    use rstest::rstest;
    use rstest_reuse::*;
    use rust_decimal_macros::dec;

    #[template]
    #[rstest]
    #[case("100.50 USD", dec!(100.50), "USD", None, None)]
    #[case("10 STOCK {50.00 USD}", dec!(10), "STOCK", Some((dec!(50.00), "USD")), None)]
    #[case("10 STOCK @ 55.00 USD", dec!(10), "STOCK", None, Some((dec!(55.00), "USD")))]
    #[case("10 STOCK {50.00 USD} @ 55.00 USD", dec!(10), "STOCK", Some((dec!(50.00), "USD")), Some((dec!(55.00), "USD")))]
    #[case("10 STOCK { 50.00 USD } @ 55.00 USD", dec!(10), "STOCK", Some((dec!(50.00), "USD")), Some((dec!(55.00), "USD")))]
    #[case("-37.45 USD", dec!(-37.45), "USD", None, None)]
    #[case("0 USD", dec!(0), "USD", None, None)]
    fn valid_posting_amount_template(
        #[case] input: &str,
        #[case] expected_number: rust_decimal::Decimal,
        #[case] expected_commodity: &str,
        #[case] expected_cost: Option<(rust_decimal::Decimal, &str)>,
        #[case] expected_price: Option<(rust_decimal::Decimal, &str)>
    ) {}

    #[apply(valid_posting_amount_template)]
    fn parse_valid_posting_amount(
        #[case] input: &str,
        #[case] expected_number: rust_decimal::Decimal,
        #[case] expected_commodity: &str,
        #[case] expected_cost: Option<(rust_decimal::Decimal, &str)>,
        #[case] expected_price: Option<(rust_decimal::Decimal, &str)>
    ) {
        let result = parse_posting_amount().parse(input);
        assert!(
            result.has_output(),
            "Failed to parse posting amount: {}",
            input
        );
        let parsed = result.into_result().unwrap();
        
        // Validate amount
        assert_eq!(*parsed.amount().number(), expected_number);
        assert_eq!(parsed.amount().commodity().as_ref(), expected_commodity);
        
        // Validate cost
        match expected_cost {
            Some((cost_number, cost_commodity)) => {
                assert!(parsed.has_cost());
                let cost = parsed.cost().unwrap();
                assert_eq!(*cost.number(), cost_number);
                assert_eq!(cost.commodity().as_ref(), cost_commodity);
            }
            None => {
                assert!(!parsed.has_cost());
            }
        }
        
        // Validate price
        match expected_price {
            Some((price_number, price_commodity)) => {
                assert!(parsed.has_price());
                let price = parsed.price().unwrap();
                assert_eq!(*price.number(), price_number);
                assert_eq!(price.commodity().as_ref(), price_commodity);
            }
            None => {
                assert!(!parsed.has_price());
            }
        }
    }

    #[apply(valid_posting_amount_template)]
    fn marshal_and_parse_posting_amount(
        #[case] input: &str,
        #[case] _expected_number: rust_decimal::Decimal,
        #[case] _expected_commodity: &str,
        #[case] _expected_cost: Option<(rust_decimal::Decimal, &str)>,
        #[case] _expected_price: Option<(rust_decimal::Decimal, &str)>
    ) {
        // First parse the original
        let result = parse_posting_amount().parse(input);
        assert!(result.has_output());
        let original = result.into_result().unwrap();

        // Marshal it
        let mut marshalled = String::new();
        let marshal_result = marshal_posting_amount(&original, &mut marshalled);
        assert!(marshal_result.is_ok());

        // Parse it back
        let reparse_result = parse_posting_amount().parse(&marshalled);
        assert!(reparse_result.has_output());
        let reparsed = reparse_result.into_result().unwrap();

        // Should be equal
        assert_eq!(original, reparsed);
    }


    #[test]
    fn marshal_posting_amount_basic() {
        let commodity = commodity!(USD);
        let amount = Amount::new(dec!(100.50), commodity);
        let posting_amount = PostingAmount::new(amount);

        let mut output = String::new();
        let result = marshal_posting_amount(&posting_amount, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "100.50 USD");
    }

    #[test]
    fn marshal_posting_amount_with_cost() {
        let stock = commodity!(STOCK);
        let usd = commodity!(USD);
        let amount = Amount::new(dec!(10), stock);
        let cost = Amount::new(dec!(50.00), usd);
        let posting_amount = PostingAmount::new(amount).with_cost(cost);

        let mut output = String::new();
        let result = marshal_posting_amount(&posting_amount, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "10 STOCK {50.00 USD}");
    }

    #[test]
    fn marshal_posting_amount_with_price() {
        let stock = commodity!(STOCK);
        let usd = commodity!(USD);
        let amount = Amount::new(dec!(10), stock);
        let price = Amount::new(dec!(55.00), usd);
        let posting_amount = PostingAmount::new(amount).with_price(price);

        let mut output = String::new();
        let result = marshal_posting_amount(&posting_amount, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "10 STOCK @ 55.00 USD");
    }

    #[test]
    fn marshal_posting_amount_with_cost_and_price() {
        let stock = commodity!(STOCK);
        let usd = commodity!(USD);
        let amount = Amount::new(dec!(10), stock);
        let cost = Amount::new(dec!(50.00), usd.clone());
        let price = Amount::new(dec!(55.00), usd);
        let posting_amount = PostingAmount::new(amount).with_cost(cost).with_price(price);

        let mut output = String::new();
        let result = marshal_posting_amount(&posting_amount, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, "10 STOCK {50.00 USD} @ 55.00 USD");
    }

    #[rstest]
    #[case("100.50")] // Missing commodity
    #[case("STOCK {50.00 USD}")] // Missing amount number
    #[case("10 STOCK {50.00}")] // Missing cost commodity
    #[case("10 STOCK @ 55.00")] // Missing price commodity
    #[case("10 STOCK {50.00 USD")] // Unclosed cost brace
    #[case("10 STOCK 50.00 USD}")] // Missing opening cost brace
    #[case("10 STOCK @")] // Missing price amount
    fn parse_posting_amount_invalid(#[case] input: &str) {
        let result = parse_posting_amount().parse(input);
        assert!(!result.has_output(), "Should fail to parse: {}", input);
    }
}
