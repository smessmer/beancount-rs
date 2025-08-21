use crate::{model::AmountWithTolerance, parser::lima::error::LimaConversionError};

impl<'a, 'r> TryFrom<&'r beancount_parser_lima::AmountWithTolerance<'a>> for AmountWithTolerance<'a>
where
    'r: 'a,
{
    type Error = LimaConversionError<'a>;

    fn try_from(
        amount: &'r beancount_parser_lima::AmountWithTolerance<'a>,
    ) -> Result<Self, Self::Error> {
        Ok(AmountWithTolerance::new(
            amount.amount().item().try_into()?,
            amount.tolerance().map(|t| *t.item()),
        ))
    }
}
