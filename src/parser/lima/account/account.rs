use crate::{
    model::{Account, AccountComponent},
    parser::lima::error::LimaConversionError,
};

impl<'a, 'r> TryFrom<&'r beancount_parser_lima::Account<'a>> for Account<'a>
where
    'r: 'a,
{
    type Error = LimaConversionError<'a>;

    fn try_from(account: &'r beancount_parser_lima::Account<'a>) -> Result<Self, Self::Error> {
        let account_type = account.account_type().into();
        let account_components = account
            .as_ref()
            .split(':')
            .skip(1)
            .map(|component| {
                AccountComponent::new(component)
                    .map_err(LimaConversionError::InvalidAccountComponent)
            })
            .collect::<Result<Vec<AccountComponent<'a>>, LimaConversionError<'a>>>()?;
        Ok(Account::new(account_type, account_components))
    }
}

#[cfg(test)]
mod tests {
    use beancount_parser_lima::{BeancountParser, BeancountSources, DirectiveVariant};

    use crate::model::account;

    use super::*;

    #[test]
    fn test_try_from() {
        let beancount_file =
            // TODO Remove newline at end after https://github.com/tesujimath/beancount-parser-lima/issues/32 is fixed
            "2020-01-01 * \"Test Transaction\"\n  Assets:US:Bank  100.00 USD\n  Expenses:Food\n";
        let beancount_file = BeancountSources::try_from(beancount_file).unwrap();
        let parser = BeancountParser::new(&beancount_file);
        let parsed = parser.parse().unwrap();
        let DirectiveVariant::Transaction(parsed_directive) =
            parsed.directives.first().unwrap().item().variant()
        else {
            panic!("Expected a transaction directive");
        };
        let parsed_account_1 = parsed_directive.postings().next().unwrap().account().item();
        let parsed_account_2 = parsed_directive
            .postings()
            .skip(1)
            .next()
            .unwrap()
            .account()
            .item();
        let account_1 = Account::try_from(parsed_account_1).unwrap();
        let account_2 = Account::try_from(parsed_account_2).unwrap();
        assert_eq!(account!(Assets:US:Bank), account_1);
        assert_eq!(account!(Expenses:Food), account_2);
    }
}
