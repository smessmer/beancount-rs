use crate::model::{Account, Flag, directive::PostingAmount};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Posting<'a, 'c> {
    account: Account<'a>,
    flag: Option<Flag>,
    amount: Option<PostingAmount<'c>>,
}

impl<'a, 'c> Posting<'a, 'c> {
    pub fn new(account: Account<'a>, amount: PostingAmount<'c>) -> Self {
        Self {
            account,
            flag: None,
            amount: Some(amount),
        }
    }

    pub fn new_without_amount(account: Account<'a>) -> Self {
        Self {
            account,
            flag: None,
            amount: None,
        }
    }

    pub fn with_flag(mut self, flag: Flag) -> Self {
        self.flag = Some(flag);
        self
    }

    pub fn account(&self) -> &Account<'a> {
        &self.account
    }

    pub fn flag(&self) -> Option<Flag> {
        self.flag
    }

    pub fn amount(&self) -> Option<&PostingAmount<'c>> {
        self.amount.as_ref()
    }

    pub fn has_amount(&self) -> bool {
        self.amount.is_some()
    }

    pub fn has_flag(&self) -> bool {
        self.flag.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Amount, account, commodity};
    use rust_decimal_macros::dec;

    #[test]
    fn test_new_posting_with_amount() {
        let account = account!(Expenses:Restaurant);
        let commodity = commodity!(USD);
        let amount = Amount::new(dec!(37.45), commodity);
        let posting_amount = PostingAmount::new(amount.clone());
        let posting = Posting::new(account.clone(), posting_amount.clone());

        assert_eq!(posting.account(), &account);
        assert_eq!(posting.amount(), Some(&posting_amount));
        assert_eq!(posting.amount().unwrap().amount(), &amount);
        assert!(posting.has_amount());
        assert!(!posting.has_flag());
    }

    #[test]
    fn test_new_posting_without_amount() {
        let account = account!(Expenses:Restaurant);
        let posting = Posting::new_without_amount(account.clone());

        assert_eq!(posting.account(), &account);
        assert_eq!(posting.amount(), None);
        assert!(!posting.has_amount());
        assert!(!posting.has_flag());
    }

    #[test]
    fn test_posting_with_negative_amount() {
        let account = account!(Liabilities:CreditCard);
        let commodity = commodity!(USD);
        let amount = Amount::new(dec!(-37.45), commodity);
        let posting_amount = PostingAmount::new(amount.clone());
        let posting = Posting::new(account.clone(), posting_amount.clone());

        assert_eq!(posting.account(), &account);
        assert_eq!(posting.amount(), Some(&posting_amount));
        assert_eq!(*posting.amount().unwrap().amount().number(), dec!(-37.45));
    }

    #[test]
    fn test_posting_equality() {
        let account1 = account!(Assets:Checking);
        let account2 = account!(Assets:Checking);
        let commodity = commodity!(USD);
        let amount1 = Amount::new(dec!(100.00), commodity.clone());
        let amount2 = Amount::new(dec!(100.00), commodity);
        let posting_amount1 = PostingAmount::new(amount1);
        let posting_amount2 = PostingAmount::new(amount2);

        let posting1 = Posting::new(account1, posting_amount1);
        let posting2 = Posting::new(account2, posting_amount2);

        assert_eq!(posting1, posting2);
    }

    #[test]
    fn test_posting_inequality_different_accounts() {
        let account1 = account!(Assets:Checking);
        let account2 = account!(Assets:Savings);
        let commodity = commodity!(USD);
        let amount = Amount::new(dec!(100.00), commodity);
        let posting_amount1 = PostingAmount::new(amount.clone());
        let posting_amount2 = PostingAmount::new(amount);

        let posting1 = Posting::new(account1, posting_amount1);
        let posting2 = Posting::new(account2, posting_amount2);

        assert_ne!(posting1, posting2);
    }

    #[test]
    fn test_posting_inequality_different_amounts() {
        let account = account!(Assets:Checking);
        let commodity = commodity!(USD);
        let amount1 = Amount::new(dec!(100.00), commodity.clone());
        let amount2 = Amount::new(dec!(200.00), commodity);
        let posting_amount1 = PostingAmount::new(amount1);
        let posting_amount2 = PostingAmount::new(amount2);

        let posting1 = Posting::new(account.clone(), posting_amount1);
        let posting2 = Posting::new(account, posting_amount2);

        assert_ne!(posting1, posting2);
    }

    #[test]
    fn test_posting_inequality_amount_vs_no_amount() {
        let account = account!(Assets:Checking);
        let commodity = commodity!(USD);
        let amount = Amount::new(dec!(100.00), commodity);
        let posting_amount = PostingAmount::new(amount);

        let posting1 = Posting::new(account.clone(), posting_amount);
        let posting2 = Posting::new_without_amount(account);

        assert_ne!(posting1, posting2);
    }

    #[test]
    fn test_clone_posting() {
        let account = account!(Income:Salary);
        let commodity = commodity!(USD);
        let amount = Amount::new(dec!(-5000.00), commodity);
        let posting_amount = PostingAmount::new(amount);
        let posting1 = Posting::new(account, posting_amount);
        let posting2 = posting1.clone();

        assert_eq!(posting1, posting2);
    }

    #[test]
    fn test_posting_with_flag() {
        let account = account!(Assets:Checking);
        let commodity = commodity!(USD);
        let amount = Amount::new(dec!(100.00), commodity);
        let posting_amount = PostingAmount::new(amount);
        let posting = Posting::new(account.clone(), posting_amount).with_flag(Flag::Complete);

        assert_eq!(posting.account(), &account);
        assert!(posting.has_flag());
        assert_eq!(posting.flag(), Some(Flag::Complete));
    }

    #[test]
    fn test_posting_amount_with_cost() {
        let account = account!(Assets:Investments);
        let stock = commodity!(STOCK);
        let usd = commodity!(USD);
        let amount = Amount::new(dec!(10), stock);
        let cost = Amount::new(dec!(50.00), usd);
        let posting_amount = PostingAmount::new(amount.clone()).with_cost(cost.clone());
        let posting = Posting::new(account, posting_amount.clone());

        assert_eq!(posting.amount(), Some(&posting_amount));
        assert_eq!(posting.amount().unwrap().amount(), &amount);
        assert_eq!(posting.amount().unwrap().cost(), Some(&cost));
        assert!(!posting.amount().unwrap().has_price());
    }

    #[test]
    fn test_posting_amount_with_price() {
        let account = account!(Assets:Investments);
        let stock = commodity!(STOCK);
        let usd = commodity!(USD);
        let amount = Amount::new(dec!(10), stock);
        let price = Amount::new(dec!(55.00), usd);
        let posting_amount = PostingAmount::new(amount.clone()).with_price(price.clone());
        let posting = Posting::new(account, posting_amount.clone());

        assert_eq!(posting.amount(), Some(&posting_amount));
        assert_eq!(posting.amount().unwrap().amount(), &amount);
        assert_eq!(posting.amount().unwrap().price(), Some(&price));
        assert!(!posting.amount().unwrap().has_cost());
    }
}
