use anyhow::Result;

use crate::model::account::{account_component::AccountComponent, account_type::AccountType};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Account<'a> {
    account_type: AccountType,
    components: Vec<AccountComponent<'a>>,
}

impl<'a> Account<'a> {
    pub fn new(account_type: AccountType, components: Vec<AccountComponent<'a>>) -> Self {
        Self {
            account_type,
            components,
        }
    }

    pub fn account_type(&self) -> AccountType {
        self.account_type
    }

    pub fn components(&self) -> impl Iterator<Item = &'_ AccountComponent<'a>> + ExactSizeIterator {
        self.components.iter().map(|c| c)
    }
}

/// Macro to create a new account with the specified type and components.
///
/// # Example
/// ```
/// use beancount_rs::model::account;
///
/// let acc = account!(Assets:US:Cash);
/// ```
#[macro_export]
macro_rules! account_ {
    ($acctype:ident : $($component:ident):*) => {
        $crate::model::Account::new($crate::model::AccountType::$acctype, vec![$($crate::model::AccountComponent::new(stringify!($component)).unwrap()),*])
    };
}
pub use account_ as account;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_creation() {
        let _acc: Account = account!(Assets:Cash);
        assert_eq!(_acc.account_type(), AccountType::Assets);
        let components: Vec<&str> = _acc.components().map(AsRef::as_ref).collect();
        assert_eq!(components, ["Cash"]);
    }

    #[test]
    #[should_panic(expected = "InvalidStart")]
    fn test_account_creation_invalid_component() {
        let _acc: Account = account!(Assets:cash);
    }

    #[test]
    fn test_account_creation_multiple_components() {
        let _acc: Account = account!(Expenses:Food:Groceries:Store);
        assert_eq!(_acc.account_type(), AccountType::Expenses);
        let components: Vec<&str> = _acc.components().map(AsRef::as_ref).collect();
        assert_eq!(components, ["Food", "Groceries", "Store"]);
    }

    #[test]
    #[should_panic(expected = "InvalidCharacter")]
    fn test_account_creation_mixed_valid_invalid() {
        let _acc: Account = account!(Income:Salary:In_valid);
    }
}
