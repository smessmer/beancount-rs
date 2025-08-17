use std::fmt::Write;

use chumsky::prelude::*;

use crate::{
    model::AccountType, parser::chumsky::account::account_component::parse_account_component,
};

const fn account_type_str(account_type: AccountType) -> &'static str {
    match account_type {
        AccountType::Assets => "Assets",
        AccountType::Liabilities => "Liabilities",
        AccountType::Income => "Income",
        AccountType::Expenses => "Expenses",
        AccountType::Equity => "Equity",
    }
}

pub fn parse_account_type<'a>() -> impl Parser<'a, &'a str, AccountType, extra::Err<Rich<'a, char>>>
{
    parse_account_component().try_map(|s, span| match s.as_ref() {
        c if c == account_type_str(AccountType::Assets) => Ok(AccountType::Assets),
        c if c == account_type_str(AccountType::Liabilities) => Ok(AccountType::Liabilities),
        c if c == account_type_str(AccountType::Income) => Ok(AccountType::Income),
        c if c == account_type_str(AccountType::Expenses) => Ok(AccountType::Expenses),
        c if c == account_type_str(AccountType::Equity) => Ok(AccountType::Equity),
        _ => Err(chumsky::error::Rich::custom(
            span,
            "Expected Assets, Liabilities, Income, Expenses or Equity",
        )),
    })
}

pub fn marshal_account_type(
    account_type: AccountType,
    writer: &mut impl Write,
) -> std::fmt::Result {
    write!(writer, "{}", account_type_str(account_type))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use rstest_reuse::*;

    mod account_type {
        use super::*;

        #[template]
        fn account_type_template(
            #[values(
                AccountType::Assets,
                AccountType::Liabilities,
                AccountType::Income,
                AccountType::Expenses,
                AccountType::Equity
            )]
            expected: AccountType,
        ) {
        }

        #[apply(account_type_template)]
        #[rstest]
        fn parse(expected: AccountType) {
            let input = account_type_str(expected);
            let result = parse_account_type().parse(input);
            assert_eq!(Ok(expected), result.into_result());
        }

        #[apply(account_type_template)]
        #[rstest]
        fn parse_with_extra_suffix(expected: AccountType) {
            let input = format!("{} extra", account_type_str(expected));
            let result = parse_account_type().parse(&input);
            assert!(result.into_result().is_err());
        }

        #[apply(account_type_template)]
        #[rstest]
        fn marshal(expected: AccountType) {
            let mut output = String::new();
            let result = marshal_account_type(expected, &mut output);
            assert!(result.is_ok());
            assert_eq!(output, account_type_str(expected));
        }

        #[apply(account_type_template)]
        #[rstest]
        fn marshal_and_parse(expected: AccountType) {
            let mut marshalled = String::new();
            marshal_account_type(expected, &mut marshalled).unwrap();

            let result = parse_account_type().parse(&marshalled);
            assert_eq!(Ok(expected), result.into_result());
        }
    }
}
