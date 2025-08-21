use thiserror::Error;

use crate::{
    model::{
        DirectiveTransaction,
        directive::{Posting, TransactionDescription},
    },
    parser::lima::error::LimaConversionError,
};

#[derive(Error, Debug)]
pub enum LimaTransactionConversionError<'a> {
    #[error("Payee specified but no narration")]
    PayeeWithoutNarration(beancount_parser_lima::Transaction<'a>),
}

impl<'a, 'r> TryFrom<&'r beancount_parser_lima::Transaction<'a>> for DirectiveTransaction<'a>
where
    'r: 'a,
{
    type Error = LimaConversionError<'a>;

    fn try_from(
        transaction: &'r beancount_parser_lima::Transaction<'a>,
    ) -> Result<Self, Self::Error> {
        let flag = transaction.flag().item().into();
        let mut result = DirectiveTransaction::new(flag);
        match (transaction.payee(), transaction.narration()) {
            (Some(payee), Some(narration)) => {
                result = result.with_description(TransactionDescription::new_with_payee(
                    *payee.item(),
                    *narration.item(),
                ))
            }
            (None, Some(narration)) => {
                result = result
                    .with_description(TransactionDescription::new_without_payee(*narration.item()))
            }
            (None, None) => (),
            (Some(_payee), None) => {
                return Err(LimaConversionError::InvalidTransaction(
                    LimaTransactionConversionError::PayeeWithoutNarration(transaction.clone()),
                ));
            }
        };
        let postings = transaction
            .postings()
            .map(|posting| posting.item().try_into())
            .collect::<Result<Vec<Posting<'a>>, LimaConversionError<'a>>>()?;
        result = result.with_postings(postings);

        Ok(result)
    }
}
