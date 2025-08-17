use crate::model::{Account, AmountWithTolerance};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectiveBalance<'a, 'c> {
    account: Account<'a>,
    amount_with_tolerance: AmountWithTolerance<'c>,
}

impl<'a, 'c> DirectiveBalance<'a, 'c> {
    pub fn new(account: Account<'a>, amount_with_tolerance: AmountWithTolerance<'c>) -> Self {
        Self {
            account,
            amount_with_tolerance,
        }
    }

    pub fn account(&self) -> &Account<'a> {
        &self.account
    }

    pub fn amount_with_tolerance(&self) -> &AmountWithTolerance<'c> {
        &self.amount_with_tolerance
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{account, commodity};
    use rust_decimal_macros::dec;

    #[test]
    fn test_new_balance_directive() {
        let account = account!(Assets:Checking);
        let commodity = commodity!(USD);
        let amount_with_tolerance =
            AmountWithTolerance::without_tolerance(dec!(1000.50), commodity);
        let balance = DirectiveBalance::new(account.clone(), amount_with_tolerance.clone());

        assert_eq!(balance.account(), &account);
        assert_eq!(balance.amount_with_tolerance(), &amount_with_tolerance);
        assert_eq!(balance.amount_with_tolerance().tolerance(), None);
    }

    #[test]
    fn test_balance_directive_with_tolerance() {
        let account = account!(Assets:Investment);
        let commodity = commodity!(RGAGX);
        let amount_with_tolerance =
            AmountWithTolerance::with_tolerance(dec!(319.020), dec!(0.002), commodity);
        let balance = DirectiveBalance::new(account.clone(), amount_with_tolerance.clone());

        assert_eq!(balance.account(), &account);
        assert_eq!(balance.amount_with_tolerance(), &amount_with_tolerance);
        assert_eq!(
            balance.amount_with_tolerance().tolerance(),
            Some(&dec!(0.002))
        );
    }

    #[test]
    fn test_negative_balance() {
        let account = account!(Liabilities:CreditCard);
        let commodity = commodity!(USD);
        let amount_with_tolerance =
            AmountWithTolerance::without_tolerance(dec!(-3492.02), commodity);
        let balance = DirectiveBalance::new(account.clone(), amount_with_tolerance.clone());

        assert_eq!(balance.account(), &account);
        assert_eq!(balance.amount_with_tolerance(), &amount_with_tolerance);
        assert_eq!(balance.amount_with_tolerance().tolerance(), None);
    }

    #[test]
    fn test_zero_balance() {
        let account = account!(Assets:Checking);
        let commodity = commodity!(USD);
        let amount_with_tolerance = AmountWithTolerance::without_tolerance(dec!(0), commodity);
        let balance = DirectiveBalance::new(account.clone(), amount_with_tolerance.clone());

        assert_eq!(balance.account(), &account);
        assert_eq!(balance.amount_with_tolerance(), &amount_with_tolerance);
        assert_eq!(balance.amount_with_tolerance().tolerance(), None);
    }

    #[test]
    fn test_clone_and_equality() {
        let account = account!(Assets:Savings);
        let commodity = commodity!(EUR);
        let amount_with_tolerance =
            AmountWithTolerance::without_tolerance(dec!(5000.00), commodity);
        let balance1 = DirectiveBalance::new(account, amount_with_tolerance);
        let balance2 = balance1.clone();

        assert_eq!(balance1, balance2);
    }

    #[test]
    fn test_balance_with_tolerance_equality() {
        let account = account!(Assets:Investment);
        let commodity = commodity!(SHARES);
        let amount_with_tolerance =
            AmountWithTolerance::with_tolerance(dec!(100.5), dec!(0.1), commodity);

        let balance1 = DirectiveBalance::new(account.clone(), amount_with_tolerance.clone());
        let balance2 = DirectiveBalance::new(account, amount_with_tolerance);

        assert_eq!(balance1, balance2);
    }

    #[test]
    fn test_different_tolerances_not_equal() {
        let account = account!(Assets:Investment);
        let commodity = commodity!(SHARES);

        let amount_with_tolerance1 =
            AmountWithTolerance::with_tolerance(dec!(100.5), dec!(0.1), commodity.clone());
        let amount_with_tolerance2 =
            AmountWithTolerance::with_tolerance(dec!(100.5), dec!(0.2), commodity);

        let balance1 = DirectiveBalance::new(account.clone(), amount_with_tolerance1);
        let balance2 = DirectiveBalance::new(account, amount_with_tolerance2);

        assert_ne!(balance1, balance2);
    }

    #[test]
    fn test_tolerance_vs_no_tolerance_not_equal() {
        let account = account!(Assets:Investment);
        let commodity = commodity!(SHARES);

        let amount_with_tolerance1 =
            AmountWithTolerance::without_tolerance(dec!(100.5), commodity.clone());
        let amount_with_tolerance2 =
            AmountWithTolerance::with_tolerance(dec!(100.5), dec!(0.1), commodity);

        let balance1 = DirectiveBalance::new(account.clone(), amount_with_tolerance1);
        let balance2 = DirectiveBalance::new(account, amount_with_tolerance2);

        assert_ne!(balance1, balance2);
    }
}
