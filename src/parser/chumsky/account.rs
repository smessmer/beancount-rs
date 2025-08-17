use chumsky::prelude::*;
use std::fmt::Write;

use crate::{
    model::{Account, AccountType},
    parser::chumsky::account_type::{marshal_account_type, parse_account_type},
};

// pub fn parse_account<'a>() -> impl Parser<'a, &'a str, AccountType> {
//     parse_account_type()
//         .then(
//             just(':')
//                 .ignore_then(take_while1(|c: char| {
//                     c.is_alphanumeric() || c == '_' || c == '-'
//                 }))
//                 .repeated(),
//         )
//         .map(|(account_type, components)| Account {
//             account_type,
//             components,
//         })
// }

pub fn marshal_account(account: Account, writer: &mut impl Write) -> std::fmt::Result {
    marshal_account_type(account.account_type(), writer)?;
    for component in account.components() {
        write!(writer, ":{}", component)?;
    }
    Ok(())
}
