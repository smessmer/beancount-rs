use anyhow::Result;

use crate::model::account::{account_component::AccountComponent, account_type::AccountType};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Account<'a> {
    account_type: AccountType,
    components: Vec<AccountComponent<'a>>,
}

impl<'a> Account<'a> {
    pub fn new<E>(
        account_type: AccountType,
        components: impl IntoIterator<Item = impl TryInto<AccountComponent<'a>, Error = E>>,
    ) -> Result<Self, E> {
        let components: Result<Vec<AccountComponent<'a>>, E> =
            components.into_iter().map(TryInto::try_into).collect();
        let components = components?;
        let res = Self {
            account_type,
            components,
        };
        Ok(res)
    }

    pub fn account_type(&self) -> AccountType {
        self.account_type
    }

    pub fn components(&self) -> impl Iterator<Item = &'_ AccountComponent<'a>> + ExactSizeIterator {
        self.components.iter().map(|c| c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_creation() {
        let account = Account::new(AccountType::Assets, ["Cash"]);
        assert!(account.is_ok());
    }

    #[test]
    fn test_account_creation_invalid_component() {
        let account = Account::new(AccountType::Assets, ["cash"]);
        assert!(account.is_err());
    }

    #[test]
    fn test_account_creation_multiple_components() {
        let account = Account::new(AccountType::Expenses, ["Food", "Groceries", "Store-1"]);
        assert!(account.is_ok());
    }

    #[test]
    fn test_account_creation_mixed_valid_invalid() {
        let account = Account::new(AccountType::Income, ["Salary", "invalid"]);
        assert!(account.is_err());
    }

    #[test]
    fn test_account_getters() {
        let account = Account::new(AccountType::Liabilities, ["CreditCard"])
            .expect("Account creation should succeed");

        assert_eq!(account.account_type(), AccountType::Liabilities);
        let components: Vec<&str> = account.components().map(AsRef::as_ref).collect();
        assert_eq!(components, ["CreditCard"]);
    }
}
