use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TransactionDescription<'a> {
    narration: Cow<'a, str>,
    payee: Option<Cow<'a, str>>,
}

impl<'a> TransactionDescription<'a> {
    pub fn new(payee: Option<impl Into<Cow<'a, str>>>, narration: impl Into<Cow<'a, str>>) -> Self {
        Self {
            narration: narration.into(),
            payee: payee.map(|p| p.into()),
        }
    }

    /// Create a new TransactionDescription with just narration
    pub fn new_without_payee(narration: impl Into<Cow<'a, str>>) -> Self {
        Self {
            narration: narration.into(),
            payee: None,
        }
    }

    /// Create a new TransactionDescription with both payee and narration
    pub fn new_with_payee(
        payee: impl Into<Cow<'a, str>>,
        narration: impl Into<Cow<'a, str>>,
    ) -> Self {
        Self {
            narration: narration.into(),
            payee: Some(payee.into()),
        }
    }

    /// Add a payee to an existing TransactionDescription
    pub fn with_payee(mut self, payee: impl Into<Cow<'a, str>>) -> Self {
        self.payee = Some(payee.into());
        self
    }

    /// Get the narration
    pub fn narration(&self) -> &str {
        self.narration.as_ref()
    }

    /// Get the payee, if any
    pub fn payee(&self) -> Option<&str> {
        self.payee.as_ref().map(|cow| cow.as_ref())
    }

    /// Check if this description has a payee
    pub fn has_payee(&self) -> bool {
        self.payee.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_transaction_description() {
        let description = TransactionDescription::new_without_payee("Direct deposit".to_string());

        assert_eq!(description.narration(), "Direct deposit");
        assert_eq!(description.payee(), None);
        assert!(!description.has_payee());
    }

    #[test]
    fn test_transaction_description_with_payee() {
        let description = TransactionDescription::new_with_payee(
            "Cafe Mogador".to_string(),
            "Lamb tagine with wine".to_string(),
        );

        assert_eq!(description.narration(), "Lamb tagine with wine");
        assert_eq!(description.payee(), Some("Cafe Mogador"));
        assert!(description.has_payee());
    }

    #[test]
    fn test_set_payee() {
        let description = TransactionDescription::new_without_payee("Lunch".to_string())
            .with_payee("Restaurant ABC".to_string());

        assert_eq!(description.narration(), "Lunch");
        assert_eq!(description.payee(), Some("Restaurant ABC"));
        assert!(description.has_payee());
    }

    #[test]
    fn test_description_equality() {
        let desc1 =
            TransactionDescription::new_with_payee("Store".to_string(), "Purchase".to_string());
        let desc2 =
            TransactionDescription::new_with_payee("Store".to_string(), "Purchase".to_string());

        assert_eq!(desc1, desc2);
    }

    #[test]
    fn test_description_inequality_different_narration() {
        let desc1 = TransactionDescription::new_without_payee("Lunch".to_string());
        let desc2 = TransactionDescription::new_without_payee("Dinner".to_string());

        assert_ne!(desc1, desc2);
    }

    #[test]
    fn test_description_inequality_different_payee() {
        let desc1 =
            TransactionDescription::new_with_payee("Store A".to_string(), "Purchase".to_string());
        let desc2 =
            TransactionDescription::new_with_payee("Store B".to_string(), "Purchase".to_string());

        assert_ne!(desc1, desc2);
    }

    #[test]
    fn test_description_inequality_payee_vs_no_payee() {
        let desc1 = TransactionDescription::new_without_payee("Purchase".to_string());
        let desc2 =
            TransactionDescription::new_with_payee("Store".to_string(), "Purchase".to_string());

        assert_ne!(desc1, desc2);
    }

    #[test]
    fn test_clone_transaction_description() {
        let desc1 =
            TransactionDescription::new_with_payee("Bank".to_string(), "Transfer".to_string());
        let desc2 = desc1.clone();

        assert_eq!(desc1, desc2);
    }

    #[test]
    fn test_description_ordering() {
        let desc_a = TransactionDescription::new_without_payee("A".to_string());
        let desc_b = TransactionDescription::new_without_payee("B".to_string());

        assert!(desc_a < desc_b);
        assert!(desc_b > desc_a);
    }

    #[test]
    fn test_empty_narration() {
        let description = TransactionDescription::new_without_payee("".to_string());

        assert_eq!(description.narration(), "");
        assert_eq!(description.payee(), None);
    }

    #[test]
    fn test_empty_payee() {
        let description =
            TransactionDescription::new_with_payee("".to_string(), "Transaction".to_string());

        assert_eq!(description.narration(), "Transaction");
        assert_eq!(description.payee(), Some(""));
        assert!(description.has_payee());
    }
}
