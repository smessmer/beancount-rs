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

fn account_type<'a>() -> impl Parser<'a, &'a str, AccountType> {
    choice((
        just(account_type_str(AccountType::Assets)).to(AccountType::Assets),
        just(account_type_str(AccountType::Liabilities)).to(AccountType::Liabilities),
        just(account_type_str(AccountType::Income)).to(AccountType::Income),
        just(account_type_str(AccountType::Expenses)).to(AccountType::Expenses),
        just(account_type_str(AccountType::Equity)).to(AccountType::Equity),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use rstest_reuse::*;

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
    fn parse_account_type(expected: AccountType) {
        let input = account_type_str(expected);
        let result = account_type().parse(input);
        assert_eq!(Ok(expected), result.into_result());
    }

    #[apply(account_type_template)]
    #[rstest]
    fn parse_with_extra_suffix(expected: AccountType) {
        let input = format!("{} extra", account_type_str(expected));
        let result = account_type().parse(&input);
        assert!(result.into_result().is_err());
    }
}
