use rust_decimal::Decimal;

use crate::model::Commodity;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Amount<'c> {
    // TODO Beancount allows expressions as amounts, should we represent that? See [beancount_parser_lima].
    number: Decimal,
    commodity: Commodity<'c>,
}

impl<'c> Amount<'c> {
    pub fn new(number: Decimal, commodity: Commodity<'c>) -> Self {
        Self { number, commodity }
    }

    pub fn number(&self) -> &Decimal {
        &self.number
    }

    pub fn commodity(&self) -> &Commodity<'c> {
        &self.commodity
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::commodity;
    use rust_decimal_macros::dec;

    #[test]
    fn test_new_amount() {
        let commodity = commodity!(USD);
        let amount = Amount::new(dec!(100.50), commodity);

        assert_eq!(*amount.number(), dec!(100.50));
        assert_eq!(amount.commodity().as_ref(), "USD");
    }

    #[test]
    fn test_amount_equality() {
        let commodity1 = commodity!(USD);
        let commodity2 = commodity!(USD);
        let amount1 = Amount::new(dec!(100.00), commodity1);
        let amount2 = Amount::new(dec!(100.00), commodity2);

        assert_eq!(amount1, amount2);
    }

    #[test]
    fn test_amount_ordering() {
        let commodity = commodity!(USD);
        let amount1 = Amount::new(dec!(50.00), commodity.clone());
        let amount2 = Amount::new(dec!(100.00), commodity);

        assert!(amount1 < amount2);
    }

    #[test]
    fn test_different_commodities() {
        let usd = commodity!(USD);
        let eur = commodity!(EUR);
        let amount1 = Amount::new(dec!(100.00), usd);
        let amount2 = Amount::new(dec!(100.00), eur);

        assert_ne!(amount1, amount2);
    }

    #[test]
    fn test_negative_amount() {
        let commodity = commodity!(USD);
        let amount = Amount::new(dec!(-50.25), commodity);

        assert_eq!(*amount.number(), dec!(-50.25));
        assert_eq!(amount.commodity().as_ref(), "USD");
    }

    #[test]
    fn test_zero_amount() {
        let commodity = commodity!(BTC);
        let amount = Amount::new(dec!(0), commodity);

        assert_eq!(*amount.number(), dec!(0));
        assert_eq!(amount.commodity().as_ref(), "BTC");
    }

    #[test]
    fn test_clone_and_hash() {
        use std::collections::HashSet;

        let commodity = commodity!(ETH);
        let amount1 = Amount::new(dec!(1.5), commodity);
        let amount2 = amount1.clone();

        assert_eq!(amount1, amount2);

        let mut set = HashSet::new();
        set.insert(amount1);
        set.insert(amount2);
        assert_eq!(set.len(), 1); // Should be treated as same element
    }
}
