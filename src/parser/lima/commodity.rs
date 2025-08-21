use crate::{model::Commodity, parser::lima::error::LimaConversionError};

impl<'a, 'r> TryFrom<&'r beancount_parser_lima::Currency<'a>> for Commodity<'a>
where
    'r: 'a,
{
    type Error = LimaConversionError<'a>;

    fn try_from(commodity: &'r beancount_parser_lima::Currency<'a>) -> Result<Self, Self::Error> {
        Commodity::new(commodity.as_ref()).map_err(LimaConversionError::InvalidCommodity)
    }
}
