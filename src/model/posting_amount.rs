use crate::model::Amount;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PostingAmount<'c> {
    amount: Amount<'c>,
    // TODO I think beancount supports total cost vs per-item cost, with {} or {{}}.
    cost: Option<Amount<'c>>,
    price: Option<Amount<'c>>,
}

impl<'c> PostingAmount<'c> {
    pub fn new(amount: Amount<'c>) -> Self {
        Self {
            amount,
            cost: None,
            price: None,
        }
    }

    pub fn with_cost(mut self, cost: Amount<'c>) -> Self {
        self.cost = Some(cost);
        self
    }

    pub fn with_price(mut self, price: Amount<'c>) -> Self {
        self.price = Some(price);
        self
    }

    pub fn amount(&self) -> &Amount<'c> {
        &self.amount
    }

    pub fn cost(&self) -> Option<&Amount<'c>> {
        self.cost.as_ref()
    }

    pub fn price(&self) -> Option<&Amount<'c>> {
        self.price.as_ref()
    }

    pub fn has_cost(&self) -> bool {
        self.cost.is_some()
    }

    pub fn has_price(&self) -> bool {
        self.price.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Amount, commodity};
    use rust_decimal_macros::dec;

    #[test]
    fn test_posting_amount_basic() {
        let commodity = commodity!(USD);
        let amount = Amount::new(dec!(100.50), commodity);
        let posting_amount = PostingAmount::new(amount.clone());

        assert_eq!(posting_amount.amount(), &amount);
        assert!(!posting_amount.has_cost());
        assert!(!posting_amount.has_price());
        assert_eq!(posting_amount.cost(), None);
        assert_eq!(posting_amount.price(), None);
    }

    #[test]
    fn test_posting_amount_with_cost() {
        let usd = commodity!(USD);
        let stock = commodity!(STOCK);
        let amount = Amount::new(dec!(10), stock);
        let cost = Amount::new(dec!(50.00), usd);

        let posting_amount = PostingAmount::new(amount.clone()).with_cost(cost.clone());

        assert_eq!(posting_amount.amount(), &amount);
        assert!(posting_amount.has_cost());
        assert!(!posting_amount.has_price());
        assert_eq!(posting_amount.cost(), Some(&cost));
        assert_eq!(posting_amount.price(), None);
    }

    #[test]
    fn test_posting_amount_with_price() {
        let usd = commodity!(USD);
        let stock = commodity!(STOCK);
        let amount = Amount::new(dec!(10), stock);
        let price = Amount::new(dec!(55.00), usd);

        let posting_amount = PostingAmount::new(amount.clone()).with_price(price.clone());

        assert_eq!(posting_amount.amount(), &amount);
        assert!(!posting_amount.has_cost());
        assert!(posting_amount.has_price());
        assert_eq!(posting_amount.cost(), None);
        assert_eq!(posting_amount.price(), Some(&price));
    }

    #[test]
    fn test_posting_amount_with_cost_and_price() {
        let usd = commodity!(USD);
        let stock = commodity!(STOCK);
        let amount = Amount::new(dec!(10), stock);
        let cost = Amount::new(dec!(50.00), usd.clone());
        let price = Amount::new(dec!(55.00), usd);

        let posting_amount = PostingAmount::new(amount.clone())
            .with_cost(cost.clone())
            .with_price(price.clone());

        assert_eq!(posting_amount.amount(), &amount);
        assert!(posting_amount.has_cost());
        assert!(posting_amount.has_price());
        assert_eq!(posting_amount.cost(), Some(&cost));
        assert_eq!(posting_amount.price(), Some(&price));
    }

    #[test]
    fn test_posting_amount_equality() {
        let usd = commodity!(USD);
        let stock = commodity!(STOCK);
        let amount = Amount::new(dec!(10), stock);
        let cost = Amount::new(dec!(50.00), usd);

        let posting_amount1 = PostingAmount::new(amount.clone()).with_cost(cost.clone());
        let posting_amount2 = PostingAmount::new(amount).with_cost(cost);

        assert_eq!(posting_amount1, posting_amount2);
    }

    #[test]
    fn test_posting_amount_clone() {
        let usd = commodity!(USD);
        let stock = commodity!(STOCK);
        let amount = Amount::new(dec!(10), stock);
        let cost = Amount::new(dec!(50.00), usd);

        let posting_amount1 = PostingAmount::new(amount).with_cost(cost);
        let posting_amount2 = posting_amount1.clone();

        assert_eq!(posting_amount1, posting_amount2);
    }
}
