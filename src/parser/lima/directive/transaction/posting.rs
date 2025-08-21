use thiserror::Error;

use crate::{
    model::{
        Amount,
        directive::{Posting, PostingAmount},
    },
    parser::lima::error::LimaConversionError,
};

#[derive(Error, Debug)]
pub enum LimaPostingConversionError<'a> {
    #[error("Currency without amount in posting: {0}")]
    CurrencyWithoutAmount(beancount_parser_lima::Posting<'a>),
    #[error("Amount without currency in posting: {0}")]
    AmountWithoutCurrency(beancount_parser_lima::Posting<'a>),
    #[error("Cost without amount in posting: {0}")]
    CostWithoutAmount(beancount_parser_lima::Posting<'a>),
    #[error("Price without amount in posting: {0}")]
    PriceWithoutAmount(beancount_parser_lima::Posting<'a>),
}

impl<'a, 'r> TryFrom<&'r beancount_parser_lima::Posting<'a>> for Posting<'a>
where
    'r: 'a,
{
    type Error = LimaConversionError<'a>;

    fn try_from(posting: &'r beancount_parser_lima::Posting<'a>) -> Result<Self, Self::Error> {
        let flag = posting.flag().map(|f| f.item().into());
        let account = posting.account().item().try_into()?;
        let cost = todo!();
        let price = todo!();
        let amount = match (posting.amount(), posting.currency()) {
            (Some(amount), Some(currency)) => {
                let mut amount =
                    PostingAmount::new(Amount::new(amount.value(), currency.item().try_into()?));
                if let Some(cost) = cost {
                    amount = amount.with_cost(cost);
                }
                if let Some(price) = price {
                    amount = amount.with_price(price);
                }
                Some(amount)
            }
            (None, None) => {
                if cost.is_some() {
                    return Err(LimaConversionError::InvalidPosting(
                        LimaPostingConversionError::CostWithoutAmount(*posting),
                    ));
                }
                if price.is_some() {
                    return Err(LimaConversionError::InvalidPosting(
                        LimaPostingConversionError::PriceWithoutAmount(*posting),
                    ));
                }
                None
            }
            (None, Some(_currency)) => {
                return Err(LimaConversionError::InvalidPosting(
                    LimaPostingConversionError::CurrencyWithoutAmount(*posting),
                ));
            }
            (Some(_amount), None) => {
                return Err(LimaConversionError::InvalidPosting(
                    LimaPostingConversionError::AmountWithoutCurrency(*posting),
                ));
            }
        };

        let posting = if let Some(amount) = amount {
            Posting::new(account, amount)
        } else {
            Posting::new_without_amount(account)
        };
        if let Some(flag) = flag {
            posting = posting.with_flag(flag);
        }
        Ok(posting)
    }
}

// TODO Tests
