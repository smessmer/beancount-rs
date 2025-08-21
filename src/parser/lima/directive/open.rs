use std::collections::HashSet;

use crate::{
    model::{Commodity, DirectiveOpen},
    parser::lima::error::LimaConversionError,
};

impl<'a, 'r> TryFrom<&'r beancount_parser_lima::Open<'a>> for DirectiveOpen<'a>
where
    'r: 'a,
{
    type Error = LimaConversionError<'a>;

    fn try_from(directive: &'r beancount_parser_lima::Open<'a>) -> Result<Self, Self::Error> {
        let account = directive.account().item().try_into()?;
        let currencies = directive
            .currencies()
            .map(|c| c.item().try_into())
            .collect::<Result<HashSet<Commodity<'a>>, LimaConversionError>>()?;
        Ok(DirectiveOpen::new(account, currencies))
    }
}
