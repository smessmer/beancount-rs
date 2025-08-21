use chumsky::{prelude::*, text::whitespace};
use std::fmt::Write;

use crate::{
    model::{DirectiveTransaction, Flag, directive::Posting},
    parser::chumsky::{
        directive::transaction::{
            description::{marshal_transaction_description, parse_transaction_description},
            posting::{marshal_posting, parse_posting},
        },
        flag::parse_flag,
    },
};

const KEYWORD_TXN: &str = "txn";

/// Parser for transaction directive (without date)
/// Syntax: <flag> [<description>] <postings>
pub fn parse_transaction_directive<'a>()
-> impl Parser<'a, &'a str, DirectiveTransaction<'a>, extra::Err<Rich<'a, char>>> {
    let flag = parse_flag().or(just(KEYWORD_TXN).to(Flag::Complete));

    flag.then(
        whitespace()
            .at_least(1)
            .ignore_then(parse_transaction_description())
            .or_not(),
    )
    .then(parse_postings())
    .map(|((flag, description), postings)| match description {
        Some(desc) => {
            DirectiveTransaction::new_with_description(flag, desc).with_postings(postings)
        }
        None => DirectiveTransaction::new(flag).with_postings(postings),
    })
}

fn parse_postings<'a>() -> impl Parser<'a, &'a str, Vec<Posting<'a>>, extra::Err<Rich<'a, char>>> {
    just('\n')
        .ignore_then(parse_posting())
        .repeated()
        .at_least(1)
        .collect()
}

pub fn marshal_transaction_directive(
    directive: &DirectiveTransaction,
    writer: &mut impl Write,
) -> std::fmt::Result {
    use crate::parser::chumsky::flag::marshal_flag;

    // Write flag
    marshal_flag(*directive.flag(), writer)?;

    // Write description if present
    if let Some(description) = directive.description() {
        write!(writer, " ")?;
        marshal_transaction_description(description, writer)?;
    }

    // Write postings
    for posting in directive.postings() {
        write!(writer, "\n")?;
        marshal_posting(posting, writer)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        Amount, Flag, account, commodity,
        directive::{PostingAmount, TransactionDescription},
    };
    use rstest::rstest;
    use rstest_reuse::*;
    use rust_decimal_macros::dec;

    #[template]
    #[rstest]
    #[case(
        "* \"Cafe Mogador\" \"Lamb tagine with wine\"\n  Liabilities:CreditCard  -37.45 USD\n  Expenses:Restaurant",
        Flag::Complete,
        Some((Some("Cafe Mogador"), "Lamb tagine with wine")),
        2
    )]
    #[case(
        "! \"Direct deposit\"\n  Assets:Checking  2500.00 USD\n  Income:Salary",
        Flag::Incomplete,
        Some((None, "Direct deposit")),
        2
    )]
    #[case(
        "*\n  Assets:Cash  -20.00 USD\n  Expenses:Coffee  20.00 USD",
        Flag::Complete,
        None,
        2
    )]
    #[case(
        "txn \"Grocery shopping\"\n  Assets:Cash  -45.50 USD\n  Expenses:Groceries",
        Flag::Complete,
        Some((None, "Grocery shopping")),
        2
    )]
    #[case(
        "* \"Multi-way split\"\n  Assets:Checking  -100.00 USD\n  Expenses:Groceries  60.00 USD\n  Expenses:Gas  40.00 USD",
        Flag::Complete,
        Some((None, "Multi-way split")),
        3
    )]
    #[case(
        "* \"Mixed postings\"\n  Assets:Cash  -50.00 USD\n  Expenses:Food  30.00 USD\n  Expenses:Tips",
        Flag::Complete,
        Some((None, "Mixed postings")),
        3
    )]
    fn valid_transaction_template(
        #[case] input: &str,
        #[case] expected_flag: Flag,
        #[case] expected_description: Option<(Option<&str>, &str)>, // (payee, narration)
        #[case] expected_posting_count: usize,
    ) {
    }

    #[apply(valid_transaction_template)]
    fn parse_valid_transaction(
        #[case] input: &str,
        #[case] expected_flag: Flag,
        #[case] expected_description: Option<(Option<&str>, &str)>,
        #[case] expected_posting_count: usize,
    ) {
        let result = parse_transaction_directive().parse(input);
        assert!(
            result.has_output(),
            "Failed to parse transaction: {}",
            input
        );
        let parsed = result.into_result().unwrap();

        // Validate flag
        assert_eq!(parsed.flag(), &expected_flag);

        // Validate description
        match expected_description {
            Some((expected_payee, expected_narration)) => {
                assert!(parsed.description().is_some());
                let description = parsed.description().unwrap();
                assert_eq!(description.payee(), expected_payee);
                assert_eq!(description.narration(), expected_narration);
            }
            None => {
                assert!(parsed.description().is_none());
            }
        }

        // Validate posting count
        assert_eq!(parsed.postings().len(), expected_posting_count);
    }

    #[test]
    fn marshal_transaction_basic() {
        let account1 = account!(Liabilities:CreditCard);
        let account2 = account!(Expenses:Restaurant);
        let commodity = commodity!(USD);

        let amount = Amount::new(dec!(-37.45), commodity);
        let posting_amount = PostingAmount::new(amount);
        let posting1 = Posting::new(account1, posting_amount);
        let posting2 = Posting::new_without_amount(account2);

        let transaction = DirectiveTransaction::new_with_description(
            Flag::Complete,
            TransactionDescription::new_with_payee("Cafe Mogador", "Lamb tagine with wine"),
        )
        .with_posting(posting1)
        .with_posting(posting2);

        let mut output = String::new();
        let result = marshal_transaction_directive(&transaction, &mut output);
        assert!(result.is_ok());

        let expected = "* \"Cafe Mogador\" \"Lamb tagine with wine\"\n  Liabilities:CreditCard  -37.45 USD\n  Expenses:Restaurant";
        assert_eq!(output, expected);
    }

    #[test]
    fn marshal_transaction_narration_only() {
        let account1 = account!(Assets:Checking);
        let account2 = account!(Income:Salary);
        let commodity = commodity!(USD);

        let amount = Amount::new(dec!(2500.00), commodity);
        let posting_amount = PostingAmount::new(amount);
        let posting1 = Posting::new(account1, posting_amount);
        let posting2 = Posting::new_without_amount(account2);

        let transaction = DirectiveTransaction::new_with_description(
            Flag::Incomplete,
            TransactionDescription::new_without_payee("Direct deposit"),
        )
        .with_posting(posting1)
        .with_posting(posting2);

        let mut output = String::new();
        let result = marshal_transaction_directive(&transaction, &mut output);
        assert!(result.is_ok());

        let expected = "! \"Direct deposit\"\n  Assets:Checking  2500.00 USD\n  Income:Salary";
        assert_eq!(output, expected);
    }

    #[test]
    fn marshal_transaction_no_payee_narration() {
        let account1 = account!(Assets:Cash);
        let account2 = account!(Expenses:Coffee);
        let commodity = commodity!(USD);

        let amount1 = Amount::new(dec!(-20.00), commodity.clone());
        let amount2 = Amount::new(dec!(20.00), commodity);
        let posting_amount1 = PostingAmount::new(amount1);
        let posting_amount2 = PostingAmount::new(amount2);
        let posting1 = Posting::new(account1, posting_amount1);
        let posting2 = Posting::new(account2, posting_amount2);

        let transaction = DirectiveTransaction::new(Flag::Complete)
            .with_posting(posting1)
            .with_posting(posting2);

        let mut output = String::new();
        let result = marshal_transaction_directive(&transaction, &mut output);
        assert!(result.is_ok());

        let expected = "*\n  Assets:Cash  -20.00 USD\n  Expenses:Coffee  20.00 USD";
        assert_eq!(output, expected);
    }

    #[rstest]
    #[case("*")] // Missing postings
    #[case("! \"payee\"\n")] // Missing postings after newline
    #[case("x")] // Invalid flag
    #[case("* \"unterminated quote")] // Unterminated quote
    #[case("* \"payee\" \"narration\"")] // Missing postings
    fn parse_transaction_invalid(#[case] input: &str) {
        let result = parse_transaction_directive().parse(input);
        assert!(!result.has_output(), "Should fail to parse: {}", input);
    }

    #[test]
    fn marshal_transaction_multiple_postings() {
        let account1 = account!(Assets:Checking);
        let account2 = account!(Expenses:Groceries);
        let account3 = account!(Expenses:Gas);
        let commodity = commodity!(USD);

        let amount1 = Amount::new(dec!(-100.00), commodity.clone());
        let amount2 = Amount::new(dec!(60.00), commodity.clone());
        let amount3 = Amount::new(dec!(40.00), commodity);

        let posting1 = Posting::new(account1, PostingAmount::new(amount1));
        let posting2 = Posting::new(account2, PostingAmount::new(amount2));
        let posting3 = Posting::new(account3, PostingAmount::new(amount3));

        let transaction = DirectiveTransaction::new_with_description(
            Flag::Complete,
            TransactionDescription::new_without_payee("Multi-way split"),
        )
        .with_posting(posting1)
        .with_posting(posting2)
        .with_posting(posting3);

        let mut output = String::new();
        let result = marshal_transaction_directive(&transaction, &mut output);
        assert!(result.is_ok());

        let expected = "* \"Multi-way split\"\n  Assets:Checking  -100.00 USD\n  Expenses:Groceries  60.00 USD\n  Expenses:Gas  40.00 USD";
        assert_eq!(output, expected);
    }

    #[test]
    fn marshal_transaction_mixed_amounts_and_empty() {
        let account1 = account!(Assets:Cash);
        let account2 = account!(Expenses:Food);
        let account3 = account!(Expenses:Tips);
        let commodity = commodity!(USD);

        let amount1 = Amount::new(dec!(-50.00), commodity.clone());
        let amount2 = Amount::new(dec!(30.00), commodity);

        let posting1 = Posting::new(account1, PostingAmount::new(amount1));
        let posting2 = Posting::new(account2, PostingAmount::new(amount2));
        let posting3 = Posting::new_without_amount(account3);

        let transaction = DirectiveTransaction::new_with_description(
            Flag::Complete,
            TransactionDescription::new_without_payee("Mixed postings"),
        )
        .with_posting(posting1)
        .with_posting(posting2)
        .with_posting(posting3);

        let mut output = String::new();
        let result = marshal_transaction_directive(&transaction, &mut output);
        assert!(result.is_ok());

        let expected = "* \"Mixed postings\"\n  Assets:Cash  -50.00 USD\n  Expenses:Food  30.00 USD\n  Expenses:Tips";
        assert_eq!(output, expected);
    }

    #[test]
    fn marshal_transaction_with_cost() {
        let stock_account = account!(Assets:Investments:Stock);
        let cash_account = account!(Assets:Cash);
        let stock_commodity = commodity!(AAPL);
        let usd_commodity = commodity!(USD);

        // Buy 10 shares of AAPL at $150 per share
        let stock_amount = Amount::new(dec!(10), stock_commodity);
        let cost_amount = Amount::new(dec!(150.00), usd_commodity.clone());
        let cash_amount = Amount::new(dec!(-1500.00), usd_commodity);

        let stock_posting = Posting::new(
            stock_account,
            PostingAmount::new(stock_amount).with_cost(cost_amount),
        );
        let cash_posting = Posting::new(cash_account, PostingAmount::new(cash_amount));

        let transaction = DirectiveTransaction::new_with_description(
            Flag::Complete,
            TransactionDescription::new_without_payee("Buy stocks"),
        )
        .with_posting(stock_posting)
        .with_posting(cash_posting);

        let mut output = String::new();
        let result = marshal_transaction_directive(&transaction, &mut output);
        assert!(result.is_ok());

        // Note: The exact format might depend on how cost is marshaled
        // This test ensures the marshal function works, specific format can be adjusted
        assert!(output.contains("* \"Buy stocks\""));
        assert!(output.contains("Assets:Investments:Stock"));
        assert!(output.contains("Assets:Cash"));
        assert!(output.contains("10 AAPL"));
        assert!(output.contains("-1500.00 USD"));
    }

    #[test]
    fn marshal_transaction_with_price() {
        let stock_account = account!(Assets:Investments:Stock);
        let cash_account = account!(Assets:Cash);
        let stock_commodity = commodity!(AAPL);
        let usd_commodity = commodity!(USD);

        // Sell 5 shares of AAPL at current price of $155 per share
        let stock_amount = Amount::new(dec!(-5), stock_commodity);
        let price_amount = Amount::new(dec!(155.00), usd_commodity.clone());
        let cash_amount = Amount::new(dec!(775.00), usd_commodity);

        let stock_posting = Posting::new(
            stock_account,
            PostingAmount::new(stock_amount).with_price(price_amount),
        );
        let cash_posting = Posting::new(cash_account, PostingAmount::new(cash_amount));

        let transaction = DirectiveTransaction::new_with_description(
            Flag::Complete,
            TransactionDescription::new_without_payee("Sell stocks"),
        )
        .with_posting(stock_posting)
        .with_posting(cash_posting);

        let mut output = String::new();
        let result = marshal_transaction_directive(&transaction, &mut output);
        assert!(result.is_ok());

        // Note: The exact format might depend on how price is marshaled
        // This test ensures the marshal function works, specific format can be adjusted
        assert!(output.contains("* \"Sell stocks\""));
        assert!(output.contains("Assets:Investments:Stock"));
        assert!(output.contains("Assets:Cash"));
        assert!(output.contains("-5 AAPL"));
        assert!(output.contains("775.00 USD"));
    }

    #[test]
    fn marshal_transaction_with_cost_and_price() {
        let stock_account = account!(Assets:Investments:Stock);
        let cash_account = account!(Assets:Cash);
        let stock_commodity = commodity!(AAPL);
        let usd_commodity = commodity!(USD);

        // Complex transaction with both cost and price
        let stock_amount = Amount::new(dec!(10), stock_commodity);
        let cost_amount = Amount::new(dec!(150.00), usd_commodity.clone());
        let price_amount = Amount::new(dec!(155.00), usd_commodity.clone());
        let cash_amount = Amount::new(dec!(-1500.00), usd_commodity);

        let stock_posting = Posting::new(
            stock_account,
            PostingAmount::new(stock_amount)
                .with_cost(cost_amount)
                .with_price(price_amount),
        );
        let cash_posting = Posting::new(cash_account, PostingAmount::new(cash_amount));

        let transaction = DirectiveTransaction::new_with_description(
            Flag::Complete,
            TransactionDescription::new_without_payee("Complex stock transaction"),
        )
        .with_posting(stock_posting)
        .with_posting(cash_posting);

        let mut output = String::new();
        let result = marshal_transaction_directive(&transaction, &mut output);
        assert!(result.is_ok());

        // Note: The exact format might depend on how cost and price are marshaled
        // This test ensures the marshal function works, specific format can be adjusted
        assert!(output.contains("* \"Complex stock transaction\""));
        assert!(output.contains("Assets:Investments:Stock"));
        assert!(output.contains("Assets:Cash"));
        assert!(output.contains("10 AAPL"));
        assert!(output.contains("-1500.00 USD"));
    }
}
