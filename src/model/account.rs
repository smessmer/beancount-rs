use std::borrow::Cow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccountType {
    Assets,
    Liabilities,
    Income,
    Expenses,
    Equity,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Account<'a> {
    pub account_type: AccountType,
    pub components: Vec<Cow<'a, str>>,
}
