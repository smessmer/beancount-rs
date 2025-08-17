use std::collections::HashSet;

use crate::model::{Account, Commodity};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectiveOpen<'a, 'c> {
    account: Account<'a>,
    commodity_constraints: HashSet<Commodity<'c>>,
    // TODO booking_method: BookingMethod,
}

impl<'a, 'c> DirectiveOpen<'a, 'c> {
    pub fn new(account: Account<'a>, commodity_constraints: HashSet<Commodity<'c>>) -> Self {
        Self {
            account,
            commodity_constraints,
        }
    }

    pub fn account(&self) -> &Account<'a> {
        &self.account
    }

    pub fn commodity_constraints(
        &self,
    ) -> impl Iterator<Item = &'_ Commodity<'c>> + ExactSizeIterator {
        self.commodity_constraints.iter().map(|c| c)
    }
}
#[cfg(test)]
mod tests {
    use common_macros::hash_set;

    use super::*;
    use crate::model::account::account;

    #[test]
    fn test_new_directive_open() {
        let account = account!(Assets:Cash);
        let commodities = hash_set![
            Commodity::new("USD").unwrap(),
            Commodity::new("EUR").unwrap()
        ];
        let directive = DirectiveOpen::new(account.clone(), commodities);

        assert_eq!(*directive.account(), account);
        assert_eq!(directive.commodity_constraints().len(), 2);
    }

    #[test]
    fn test_empty_commodity_constraints() {
        let account = account!(Liabilities:CreditCard);
        let directive = DirectiveOpen::new(account, hash_set![]);

        assert_eq!(directive.commodity_constraints().len(), 0);
    }

    #[test]
    fn test_clone_and_equality() {
        let account = account!(Expenses:Food);
        let commodities = hash_set![Commodity::new("USD").unwrap()];
        let directive1 = DirectiveOpen::new(account, commodities.clone());
        let directive2 = directive1.clone();

        assert_eq!(directive1, directive2);
    }

    #[test]
    fn test_commodity_order_independence() {
        let account = account!(Assets:Checking);
        let commodities1 = hash_set![
            Commodity::new("USD").unwrap(),
            Commodity::new("EUR").unwrap(),
            Commodity::new("GBP").unwrap()
        ];
        let commodities2 = hash_set![
            Commodity::new("GBP").unwrap(),
            Commodity::new("USD").unwrap(),
            Commodity::new("EUR").unwrap()
        ];

        let directive1 = DirectiveOpen::new(account.clone(), commodities1);
        let directive2 = DirectiveOpen::new(account, commodities2);

        assert_eq!(directive1, directive2);
    }
}
