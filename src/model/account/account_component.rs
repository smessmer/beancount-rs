use derive_more::Display;
use std::borrow::Cow;
use thiserror::Error;

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InvalidAccountComponentError {
    #[error("Account component cannot be empty")]
    Empty,
    #[error("Account component must start with an uppercase letter or a number")]
    InvalidStart,
    #[error("Account component can only contain letters, numbers or dashes")]
    InvalidCharacter,
}

#[derive(Debug, Display, Clone, PartialEq, Eq, Hash)]
pub struct AccountComponent<'a> {
    component: Cow<'a, str>,
}

impl<'a> AccountComponent<'a> {
    pub fn new(component: impl Into<Cow<'a, str>>) -> Result<Self, InvalidAccountComponentError> {
        let this = Self {
            component: component.into(),
        };
        this.validate()?;
        Ok(this)
    }

    fn validate(&self) -> Result<(), InvalidAccountComponentError> {
        let mut chars = self.component.chars();

        // first character needs to be an uppercase character
        match chars.next() {
            None => return Err(InvalidAccountComponentError::Empty),
            Some(c) => {
                if c.is_uppercase() || c.is_numeric() {
                    // everything is fine
                } else {
                    return Err(InvalidAccountComponentError::InvalidStart);
                }
            }
        }

        // later characters need to be letters, numbers or dashes
        for c in chars {
            if !c.is_alphanumeric() && c != '-' {
                return Err(InvalidAccountComponentError::InvalidCharacter);
            }
        }

        Ok(())
    }
}

impl TryFrom<String> for AccountComponent<'static> {
    type Error = InvalidAccountComponentError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        AccountComponent::new(value)
    }
}

impl<'a> TryFrom<&'a str> for AccountComponent<'a> {
    type Error = InvalidAccountComponentError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        AccountComponent::new(value)
    }
}

impl<'a> AsRef<str> for AccountComponent<'a> {
    fn as_ref(&self) -> &str {
        &self.component
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_components() {
        assert!(AccountComponent::new("Assets").is_ok());
        assert!(AccountComponent::new("Liabilities").is_ok());
        assert!(AccountComponent::new("Equity").is_ok());
        assert!(AccountComponent::new("Income").is_ok());
        assert!(AccountComponent::new("Expenses").is_ok());
        assert!(AccountComponent::new("Checking").is_ok());
        assert!(AccountComponent::new("Credit-Card").is_ok());
        assert!(AccountComponent::new("401k").is_ok());
        assert!(AccountComponent::new("123Plan").is_ok());
    }

    #[test]
    fn test_empty_component() {
        let result = AccountComponent::new("");
        assert_eq!(result.unwrap_err(), InvalidAccountComponentError::Empty);
    }

    #[test]
    fn test_invalid_start_lowercase() {
        let result = AccountComponent::new("assets");
        assert_eq!(
            result.unwrap_err(),
            InvalidAccountComponentError::InvalidStart
        );
    }

    #[test]
    fn test_invalid_start_special_char() {
        let result = AccountComponent::new("-Assets");
        assert_eq!(
            result.unwrap_err(),
            InvalidAccountComponentError::InvalidStart
        );

        let result = AccountComponent::new("_Assets");
        assert_eq!(
            result.unwrap_err(),
            InvalidAccountComponentError::InvalidStart
        );
    }

    #[test]
    fn test_invalid_characters() {
        let result = AccountComponent::new("Assets_Checking");
        assert_eq!(
            result.unwrap_err(),
            InvalidAccountComponentError::InvalidCharacter
        );

        let result = AccountComponent::new("Assets@Bank");
        assert_eq!(
            result.unwrap_err(),
            InvalidAccountComponentError::InvalidCharacter
        );

        let result = AccountComponent::new("Assets Bank");
        assert_eq!(
            result.unwrap_err(),
            InvalidAccountComponentError::InvalidCharacter
        );
    }

    #[test]
    fn test_try_from_string() {
        let component: Result<AccountComponent, _> = "Assets".to_string().try_into();
        assert!(component.is_ok());

        let component: Result<AccountComponent, _> = "invalid".to_string().try_into();
        assert!(component.is_err());
    }

    #[test]
    fn test_try_from_str() {
        let component: Result<AccountComponent, _> = "Assets".try_into();
        assert!(component.is_ok());

        let component: Result<AccountComponent, _> = "invalid".try_into();
        assert!(component.is_err());
    }

    #[test]
    fn test_as_ref() {
        let component = AccountComponent::new("Assets").unwrap();
        assert_eq!(component.as_ref(), "Assets");
    }

    #[test]
    fn test_display() {
        let component = AccountComponent::new("Credit-Card").unwrap();
        assert_eq!(format!("{}", component), "Credit-Card");
    }

    #[test]
    fn test_clone_and_equality() {
        let component1 = AccountComponent::new("Assets").unwrap();
        let component2 = component1.clone();
        assert_eq!(component1, component2);
    }
}
