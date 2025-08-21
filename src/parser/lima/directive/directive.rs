use crate::{
    model::{Directive, DirectiveVariant},
    parser::lima::error::LimaConversionError,
};

impl<'a, 'r> TryFrom<&'r beancount_parser_lima::Directive<'a>> for Directive<'a>
where
    'r: 'a,
{
    type Error = LimaConversionError<'a>;

    fn try_from(directive: &'r beancount_parser_lima::Directive<'a>) -> Result<Self, Self::Error> {
        let date = date_into(directive.date().item());
        let variant = match directive.variant() {
            beancount_parser_lima::DirectiveVariant::Open(open) => {
                DirectiveVariant::Open(open.try_into()?)
            }
            beancount_parser_lima::DirectiveVariant::Transaction(transaction) => {
                DirectiveVariant::Transaction(transaction.try_into()?)
            }
            _ => todo!(),
        };
        Ok(Directive::new(date, variant))
    }
}

fn date_into(date: &time::Date) -> chrono::NaiveDate {
    // TODO Is there a more efficient way?
    chrono::NaiveDate::from_ymd_opt(date.year(), date.month() as u32, date.day() as u32)
        .expect("Invalid date conversion")
}
