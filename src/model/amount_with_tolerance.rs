use rust_decimal::Decimal;

use crate::model::{Amount, Commodity};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AmountWithTolerance<'c> {
    amount: Amount<'c>,
    tolerance: Option<Decimal>,
}

impl<'c> AmountWithTolerance<'c> {
    pub fn new(amount: Amount<'c>, tolerance: Option<Decimal>) -> Self {
        Self { amount, tolerance }
    }

    pub fn without_tolerance(number: Decimal, commodity: Commodity<'c>) -> Self {
        Self {
            amount: Amount::new(number, commodity),
            tolerance: None,
        }
    }

    pub fn with_tolerance(number: Decimal, tolerance: Decimal, commodity: Commodity<'c>) -> Self {
        Self {
            amount: Amount::new(number, commodity),
            tolerance: Some(tolerance),
        }
    }

    pub fn from_amount(amount: Amount<'c>) -> Self {
        Self {
            amount,
            tolerance: None,
        }
    }

    pub fn from_amount_with_tolerance(amount: Amount<'c>, tolerance: Decimal) -> Self {
        Self {
            amount,
            tolerance: Some(tolerance),
        }
    }

    pub fn number(&self) -> &Decimal {
        self.amount.number()
    }

    pub fn tolerance(&self) -> Option<&Decimal> {
        self.tolerance.as_ref()
    }

    pub fn commodity(&self) -> &Commodity<'c> {
        self.amount.commodity()
    }

    pub fn amount(&self) -> &Amount<'c> {
        &self.amount
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::commodity;
    use rust_decimal_macros::dec;

    #[test]
    fn test_new_amount_with_tolerance() {
        let commodity = commodity!(USD);
        let amount = AmountWithTolerance::without_tolerance(dec!(100.50), commodity);

        assert_eq!(*amount.number(), dec!(100.50));
        assert_eq!(amount.tolerance(), None);
        assert_eq!(amount.commodity().as_ref(), "USD");
    }

    #[test]
    fn test_with_tolerance() {
        let commodity = commodity!(RGAGX);
        let amount = AmountWithTolerance::with_tolerance(dec!(319.020), dec!(0.002), commodity);

        assert_eq!(*amount.number(), dec!(319.020));
        assert_eq!(amount.tolerance(), Some(&dec!(0.002)));
        assert_eq!(amount.commodity().as_ref(), "RGAGX");
    }

    #[test]
    fn test_from_amount() {
        let commodity = commodity!(BTC);
        let basic_amount = Amount::new(dec!(1.5), commodity);
        let amount_with_tolerance = AmountWithTolerance::from_amount(basic_amount);

        assert_eq!(*amount_with_tolerance.number(), dec!(1.5));
        assert_eq!(amount_with_tolerance.tolerance(), None);
        assert_eq!(amount_with_tolerance.commodity().as_ref(), "BTC");
    }

    #[test]
    fn test_to_amount() {
        let commodity = commodity!(EUR);
        let amount_with_tolerance =
            AmountWithTolerance::with_tolerance(dec!(500.00), dec!(0.10), commodity);
        let basic_amount = amount_with_tolerance.amount();

        assert_eq!(*basic_amount.number(), dec!(500.00));
        assert_eq!(basic_amount.commodity().as_ref(), "EUR");
    }

    #[test]
    fn test_equality() {
        let commodity1 = commodity!(USD);
        let commodity2 = commodity!(USD);
        let amount1 = AmountWithTolerance::with_tolerance(dec!(100.00), dec!(0.01), commodity1);
        let amount2 = AmountWithTolerance::with_tolerance(dec!(100.00), dec!(0.01), commodity2);

        assert_eq!(amount1, amount2);
    }

    #[test]
    fn test_different_tolerances() {
        let commodity = commodity!(USD);
        let amount1 =
            AmountWithTolerance::with_tolerance(dec!(100.00), dec!(0.01), commodity.clone());
        let amount2 = AmountWithTolerance::with_tolerance(dec!(100.00), dec!(0.02), commodity);

        assert_ne!(amount1, amount2);
    }

    #[test]
    fn test_tolerance_vs_no_tolerance() {
        let commodity = commodity!(USD);
        let amount1 = AmountWithTolerance::without_tolerance(dec!(100.00), commodity.clone());
        let amount2 = AmountWithTolerance::with_tolerance(dec!(100.00), dec!(0.01), commodity);

        assert_ne!(amount1, amount2);
    }

    #[test]
    fn test_clone_and_hash() {
        use std::collections::HashSet;

        let commodity = commodity!(ETH);
        let amount1 = AmountWithTolerance::with_tolerance(dec!(1.5), dec!(0.001), commodity);
        let amount2 = amount1.clone();

        assert_eq!(amount1, amount2);

        let mut set = HashSet::new();
        set.insert(amount1);
        set.insert(amount2);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_negative_amount() {
        let commodity = commodity!(USD);
        let amount = AmountWithTolerance::without_tolerance(dec!(-50.25), commodity);

        assert_eq!(*amount.number(), dec!(-50.25));
        assert_eq!(amount.tolerance(), None);
        assert_eq!(amount.commodity().as_ref(), "USD");
    }

    #[test]
    fn test_zero_tolerance() {
        let commodity = commodity!(BTC);
        let amount = AmountWithTolerance::with_tolerance(dec!(1.0), dec!(0), commodity);

        assert_eq!(*amount.number(), dec!(1.0));
        assert_eq!(amount.tolerance(), Some(&dec!(0)));
        assert_eq!(amount.commodity().as_ref(), "BTC");
    }
}
