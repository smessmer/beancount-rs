use crate::{model::DirectiveBalance, parser::lima::error::LimaConversionError};

impl<'a, 'r> TryFrom<&'r beancount_parser_lima::Balance<'a>> for DirectiveBalance<'a>
where
    'r: 'a,
{
    type Error = LimaConversionError<'a>;

    fn try_from(balance: &'r beancount_parser_lima::Balance<'a>) -> Result<Self, Self::Error> {
        Ok(DirectiveBalance::new(
            balance.account().item().try_into()?,
            balance.atol().item().try_into()?,
        ))
    }
}
