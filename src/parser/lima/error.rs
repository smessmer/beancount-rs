use thiserror::Error;

use crate::{
    model::{InvalidAccountComponentError, InvalidCommodityError},
    parser::lima::directive::{LimaPostingConversionError, LimaTransactionConversionError},
};

/// An error happened when attempting to convert the output of [beancount_parser_lima] into our model types.
#[derive(Debug, Error)]
pub enum LimaConversionError<'a> {
    #[error("Invalid commodity: {0}")]
    InvalidCommodity(#[from] InvalidCommodityError),

    #[error("Invalid account component: {0}")]
    InvalidAccountComponent(#[from] InvalidAccountComponentError),

    #[error("Invalid posting: {0}")]
    InvalidPosting(LimaPostingConversionError<'a>),

    #[error("Invalid transaction: {0}")]
    InvalidTransaction(LimaTransactionConversionError<'a>),
}
