use anyhow::Result;
use std::borrow::Cow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccountType {
    Assets,
    Liabilities,
    Income,
    Expenses,
    Equity,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Account<'a> {
    account_type: AccountType,
    components: Vec<Cow<'a, str>>,
}

impl<'a> Account<'a> {
    pub fn new(
        account_type: AccountType,
        components: impl IntoIterator<Item = impl Into<Cow<'a, str>>>,
    ) -> Result<Self> {
        let components: Vec<Cow<'a, str>> = components.into_iter().map(Into::into).collect();
        let res = Self {
            account_type,
            components,
        };
        res.validate()?;
        Ok(res)
    }

    fn validate(&self) -> Result<()> {
        for component in &self.components {
            let mut chars = component.chars();
            // first character needs to be an uppercase character
            match chars.next() {
                None => return Err(anyhow::anyhow!("Account component cannot be empty")),
                Some(c) => {
                    if c.is_numeric() {
                        // everything is fine
                    } else if c.is_alphabetic() && c.is_uppercase() {
                        // everything is fine
                    } else {
                        return Err(anyhow::anyhow!(
                            "Account component must start with an uppercase letter or a number"
                        ));
                    }
                }
            }
            // later characters need to be letters, numbers or dashes
            for c in chars {
                if !c.is_alphanumeric() && c != '-' {
                    return Err(anyhow::anyhow!(
                        "Account component can only contain letters, numbers or dashes"
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn account_type(&self) -> AccountType {
        self.account_type
    }

    pub fn components(&self) -> impl Iterator<Item = &'_ str> + ExactSizeIterator {
        self.components.iter().map(|c| c.as_ref())
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
    fn test_account_creation_valid_uppercase_start() {
        let account = Account::new(AccountType::Assets, ["Assets", "Bank"]);
        assert!(account.is_ok());
    }

    #[test]
    fn test_account_creation_valid_numeric_start() {
        let account = Account::new(AccountType::Assets, ["401k"]);
        assert!(account.is_ok());
    }

    #[test]
    fn test_account_creation_valid_with_dashes() {
        let account = Account::new(AccountType::Assets, ["Checking-account"]);
        assert!(account.is_ok());
    }

    #[test]
    fn test_account_creation_empty_component() {
        let account = Account::new(AccountType::Assets, [""]);
        assert!(account.is_err());
    }

    #[test]
    fn test_account_creation_lowercase_start() {
        let account = Account::new(AccountType::Assets, ["assets"]);
        assert!(account.is_err());
    }

    #[test]
    fn test_account_creation_special_char_start() {
        let account = Account::new(AccountType::Assets, ["_Assets"]);
        assert!(account.is_err());
    }

    #[test]
    fn test_account_creation_invalid_char_middle() {
        let account = Account::new(AccountType::Assets, ["Assets@Bank"]);
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
        let components: Vec<&str> = account.components().collect();
        assert_eq!(components, ["CreditCard"]);
    }
}
