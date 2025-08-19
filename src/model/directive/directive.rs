use chrono::NaiveDate;

use super::{DirectiveBalance, DirectiveOpen, DirectiveTransaction};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectiveContent<'a, 'c> {
    Open(DirectiveOpen<'a, 'c>),
    Balance(DirectiveBalance<'a, 'c>),
    Transaction(DirectiveTransaction<'a, 'c>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Directive<'a, 'c> {
    date: NaiveDate,
    content: DirectiveContent<'a, 'c>,
}

impl<'a, 'c> Directive<'a, 'c> {
    pub fn new(date: NaiveDate, content: DirectiveContent<'a, 'c>) -> Self {
        Self { date, content }
    }

    pub fn new_open(date: NaiveDate, open: DirectiveOpen<'a, 'c>) -> Self {
        Self::new(date, DirectiveContent::Open(open))
    }

    pub fn new_balance(date: NaiveDate, balance: DirectiveBalance<'a, 'c>) -> Self {
        Self::new(date, DirectiveContent::Balance(balance))
    }

    pub fn new_transaction(date: NaiveDate, transaction: DirectiveTransaction<'a, 'c>) -> Self {
        Self::new(date, DirectiveContent::Transaction(transaction))
    }

    pub fn date(&self) -> &NaiveDate {
        &self.date
    }

    pub fn content(&self) -> &DirectiveContent<'a, 'c> {
        &self.content
    }

    pub fn as_open(&self) -> Option<&DirectiveOpen<'a, 'c>> {
        match &self.content {
            DirectiveContent::Open(open) => Some(open),
            _ => None,
        }
    }

    pub fn into_open(self) -> Option<DirectiveOpen<'a, 'c>> {
        match self.content {
            DirectiveContent::Open(open) => Some(open),
            _ => None,
        }
    }

    pub fn as_balance(&self) -> Option<&DirectiveBalance<'a, 'c>> {
        match &self.content {
            DirectiveContent::Balance(balance) => Some(balance),
            _ => None,
        }
    }

    pub fn into_balance(self) -> Option<DirectiveBalance<'a, 'c>> {
        match self.content {
            DirectiveContent::Balance(balance) => Some(balance),
            _ => None,
        }
    }

    pub fn as_transaction(&self) -> Option<&DirectiveTransaction<'a, 'c>> {
        match &self.content {
            DirectiveContent::Transaction(transaction) => Some(transaction),
            _ => None,
        }
    }

    pub fn into_transaction(self) -> Option<DirectiveTransaction<'a, 'c>> {
        match self.content {
            DirectiveContent::Transaction(transaction) => Some(transaction),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{account, commodity};
    use chrono::NaiveDate;
    use common_macros::hash_set;

    #[test]
    fn test_new_directive_open() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let account = account!(Assets:Cash);
        let commodities = hash_set![commodity!(USD)];
        let open_directive = DirectiveOpen::new(account.clone(), commodities);

        let directive = Directive::new_open(date, open_directive);

        assert_eq!(directive.date(), &date);
        assert!(directive.as_open().is_some());
        assert_eq!(directive.as_open().unwrap().account(), &account);
    }

    #[test]
    fn test_new_directive_with_content() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let account = account!(Assets:Cash);
        let commodities = hash_set![commodity!(USD)];
        let open_directive = DirectiveOpen::new(account.clone(), commodities);
        let content = DirectiveContent::Open(open_directive);

        let directive = Directive::new(date, content);

        assert_eq!(directive.date(), &date);
        assert!(directive.as_open().is_some());
        assert_eq!(directive.as_open().unwrap().account(), &account);
    }

    #[test]
    fn test_into_open() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let account = account!(Assets:Cash);
        let commodities = hash_set![commodity!(USD)];
        let open_directive = DirectiveOpen::new(account.clone(), commodities);

        let directive = Directive::new_open(date, open_directive);
        let extracted_open = directive.into_open().unwrap();

        assert_eq!(extracted_open.account(), &account);
    }

    #[test]
    fn test_clone_and_equality() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let account = account!(Assets:Cash);
        let commodities = hash_set![commodity!(USD)];
        let open_directive = DirectiveOpen::new(account, commodities);

        let directive1 = Directive::new_open(date, open_directive);
        let directive2 = directive1.clone();

        assert_eq!(directive1, directive2);
    }

    #[test]
    fn test_different_dates_not_equal() {
        let date1 = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2024, 1, 2).unwrap();
        let account = account!(Assets:Cash);
        let commodities = hash_set![commodity!(USD)];
        let open_directive = DirectiveOpen::new(account, commodities.clone());
        let open_directive2 = DirectiveOpen::new(account!(Assets:Cash), commodities);

        let directive1 = Directive::new_open(date1, open_directive);
        let directive2 = Directive::new_open(date2, open_directive2);

        assert_ne!(directive1, directive2);
    }
}
