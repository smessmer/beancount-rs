use crate::{model::Amount, parser::lima::error::LimaConversionError};

impl<'a, 'r> TryFrom<&'r beancount_parser_lima::Amount<'a>> for Amount<'a>
where
    'r: 'a,
{
    type Error = LimaConversionError<'a>;

    fn try_from(amount: &'r beancount_parser_lima::Amount<'a>) -> Result<Self, Self::Error> {
        Ok(Amount::new(
            amount.number().value(),
            amount.currency().item().try_into()?,
        ))
    }
}
