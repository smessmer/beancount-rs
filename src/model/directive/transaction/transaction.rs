use crate::model::{
    Flag,
    directive::{Posting, transaction::TransactionDescription},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DirectiveTransaction<'a> {
    flag: Flag,
    description: Option<TransactionDescription<'a>>,
    postings: Vec<Posting<'a>>,
}

impl<'a> DirectiveTransaction<'a> {
    pub fn new(flag: Flag) -> Self {
        Self {
            flag,
            description: None,
            postings: Vec::new(),
        }
    }

    pub fn new_with_description(flag: Flag, description: TransactionDescription<'a>) -> Self {
        Self {
            flag,
            description: Some(description),
            postings: Vec::new(),
        }
    }

    pub fn flag(&self) -> &Flag {
        &self.flag
    }

    pub fn description(&self) -> Option<&TransactionDescription<'a>> {
        self.description.as_ref()
    }

    pub fn postings(&self) -> &[Posting<'a>] {
        &self.postings
    }

    pub fn add_posting(&mut self, posting: Posting<'a>) {
        self.postings.push(posting);
    }

    pub fn with_posting(mut self, posting: Posting<'a>) -> Self {
        self.add_posting(posting);
        self
    }

    pub fn with_postings(mut self, postings: Vec<Posting<'a>>) -> Self {
        self.postings = postings;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Amount, account, commodity, directive::PostingAmount};
    use rust_decimal_macros::dec;

    #[test]
    fn test_new_transaction() {
        let transaction = DirectiveTransaction::new(Flag::Complete);

        assert_eq!(transaction.flag(), &Flag::Complete);
        assert_eq!(transaction.description(), None);
        assert_eq!(transaction.postings().len(), 0);
    }

    #[test]
    fn test_transaction_with_description() {
        let transaction = DirectiveTransaction::new_with_description(
            Flag::Incomplete,
            TransactionDescription::new_with_payee("Cafe Mogador", "Lamb tagine with wine"),
        );

        assert_eq!(transaction.flag(), &Flag::Incomplete);
        assert_eq!(
            transaction.description().as_ref().and_then(|d| d.payee()),
            Some("Cafe Mogador")
        );
        assert_eq!(
            transaction.description().as_ref().map(|d| d.narration()),
            Some("Lamb tagine with wine")
        );
        assert_eq!(transaction.postings().len(), 0);
    }

    #[test]
    fn test_transaction_with_postings() {
        let account1 = account!(Liabilities:CreditCard);
        let account2 = account!(Expenses:Restaurant);
        let commodity = commodity!(USD);

        let amount1 = Amount::new(dec!(-37.45), commodity.clone());
        let amount2 = Amount::new(dec!(37.45), commodity);

        let posting_amount1 = PostingAmount::new(amount1);
        let posting_amount2 = PostingAmount::new(amount2);
        let posting1 = Posting::new(account1, posting_amount1);
        let posting2 = Posting::new(account2, posting_amount2);

        let transaction = DirectiveTransaction::new(Flag::Complete)
            .with_posting(posting1.clone())
            .with_posting(posting2.clone());

        assert_eq!(transaction.postings().len(), 2);
        assert_eq!(transaction.postings()[0], posting1);
        assert_eq!(transaction.postings()[1], posting2);
    }

    #[test]
    fn test_transaction_with_missing_amount() {
        let account1 = account!(Liabilities:CreditCard);
        let account2 = account!(Expenses:Restaurant);
        let commodity = commodity!(USD);

        let amount = Amount::new(dec!(-37.45), commodity);
        let posting_amount = PostingAmount::new(amount);
        let posting1 = Posting::new(account1, posting_amount);
        let posting2 = Posting::new_without_amount(account2);

        let transaction = DirectiveTransaction::new(Flag::Complete)
            .with_posting(posting1)
            .with_posting(posting2.clone());

        assert_eq!(transaction.postings().len(), 2);
        assert!(!transaction.postings()[1].has_amount());
        assert_eq!(transaction.postings()[1], posting2);
    }

    #[test]
    fn test_add_posting() {
        let mut transaction = DirectiveTransaction::new(Flag::Complete);
        let account = account!(Assets:Cash);
        let commodity = commodity!(USD);
        let amount = Amount::new(dec!(100.00), commodity);
        let posting_amount = PostingAmount::new(amount);
        let posting = Posting::new(account, posting_amount);

        transaction.add_posting(posting.clone());

        assert_eq!(transaction.postings().len(), 1);
        assert_eq!(transaction.postings()[0], posting);
    }

    #[test]
    fn test_transaction_with_postings_builder() {
        let account1 = account!(Assets:Checking);
        let account2 = account!(Expenses:Groceries);
        let commodity = commodity!(USD);

        let amount1 = Amount::new(dec!(-50.00), commodity.clone());
        let amount2 = Amount::new(dec!(50.00), commodity);

        let posting_amount1 = PostingAmount::new(amount1);
        let posting_amount2 = PostingAmount::new(amount2);
        let posting1 = Posting::new(account1, posting_amount1);
        let posting2 = Posting::new(account2, posting_amount2);

        let postings = vec![posting1.clone(), posting2.clone()];
        let transaction = DirectiveTransaction::new(Flag::Complete).with_postings(postings);

        assert_eq!(transaction.postings().len(), 2);
        assert_eq!(transaction.postings()[0], posting1);
        assert_eq!(transaction.postings()[1], posting2);
    }

    #[test]
    fn test_transaction_flag_equality() {
        assert_eq!(Flag::Complete, Flag::Complete);
        assert_eq!(Flag::Incomplete, Flag::Incomplete);
        assert_ne!(Flag::Complete, Flag::Incomplete);
    }

    #[test]
    fn test_clone_transaction() {
        let transaction1 = DirectiveTransaction::new_with_description(
            Flag::Complete,
            TransactionDescription::new_with_payee("Store", "Purchase"),
        );
        let transaction2 = transaction1.clone();

        assert_eq!(transaction1, transaction2);
    }

    #[test]
    fn test_transaction_only_narration() {
        let transaction = DirectiveTransaction::new_with_description(
            Flag::Complete,
            TransactionDescription::new_without_payee("Direct deposit"),
        );

        assert_eq!(transaction.description().and_then(|d| d.payee()), None);
        assert_eq!(
            transaction.description().map(|d| d.narration()),
            Some("Direct deposit")
        );
    }

    #[test]
    fn test_transaction_only_payee() {
        // In the new model, you can't have payee without narration
        // This test now checks that we can create a transaction with both payee and narration
        let description = TransactionDescription::new_with_payee(
            "Bank of America".to_string(),
            "Transfer".to_string(),
        );
        let transaction = DirectiveTransaction::new_with_description(Flag::Complete, description);

        assert_eq!(
            transaction.description().and_then(|d| d.payee()),
            Some("Bank of America")
        );
        assert_eq!(
            transaction.description().map(|d| d.narration()),
            Some("Transfer")
        );
    }
}
