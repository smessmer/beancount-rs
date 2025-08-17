use std::fmt::Write;

use chumsky::prelude::*;

use crate::model::AccountType;

const fn account_type_str(account_type: AccountType) -> &'static str {
    match account_type {
        AccountType::Assets => "Assets",
        AccountType::Liabilities => "Liabilities",
        AccountType::Income => "Income",
        AccountType::Expenses => "Expenses",
        AccountType::Equity => "Equity",
    }
}

fn parse_account_type<'a>() -> impl Parser<'a, &'a str, AccountType> {
    choice((
        just(account_type_str(AccountType::Assets)).to(AccountType::Assets),
        just(account_type_str(AccountType::Liabilities)).to(AccountType::Liabilities),
        just(account_type_str(AccountType::Income)).to(AccountType::Income),
        just(account_type_str(AccountType::Expenses)).to(AccountType::Expenses),
        just(account_type_str(AccountType::Equity)).to(AccountType::Equity),
    ))
}

fn marshal_account_type(account_type: AccountType, mut writer: impl Write) -> std::fmt::Result {
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
