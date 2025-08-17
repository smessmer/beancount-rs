use std::borrow::Cow;

use derive_more::Display;
use thiserror::Error;

const MAX_COMMODITY_NAME_LENGTH: usize = 24;

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InvalidCommodityError {
    #[error("Commodity name cannot be empty")]
    Empty,
    #[error("Commodity name must start with a capital letter")]
    InvalidStart,
    #[error(
        "Commodity name can only contain capital letters, numbers, or punctuation (apostrophe, period, underscore, dash)"
    )]
    InvalidCharacter,
    #[error("Commodity name must end with a capital letter or number")]
    InvalidEnd,
    #[error("Commodity names can only be up to {MAX_COMMODITY_NAME_LENGTH} characters long")]
    TooLong,
}

#[derive(Debug, Display, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Commodity<'a> {
    commodity: Cow<'a, str>,
}

impl<'a> Commodity<'a> {
    pub fn new(commodity: impl Into<Cow<'a, str>>) -> Result<Self, InvalidCommodityError> {
        let this = Commodity {
            commodity: commodity.into(),
        };
        this.validate()?;
        Ok(this)
    }

    fn validate(&self) -> Result<(), InvalidCommodityError> {
        if self.commodity.len() > MAX_COMMODITY_NAME_LENGTH {
            return Err(InvalidCommodityError::TooLong);
        }
        let mut chars = self.commodity.chars();
        match chars.next() {
            None => return Err(InvalidCommodityError::Empty),
            Some(c) => {
                if c.is_uppercase() {
                    // everything is fine
                } else {
                    return Err(InvalidCommodityError::InvalidStart);
                }
            }
        }
        match chars.next_back() {
            None => {
                // The whole currency is just one character. The rules for the first character as checked above are a superset of the rules for the last character, so we don't need to check it again.
            }
            Some(c) => {
                if c.is_uppercase() || c.is_numeric() {
                    // everything is fine
                } else {
                    return Err(InvalidCommodityError::InvalidEnd);
                }
            }
        }
        for c in chars {
            if !c.is_uppercase() && !c.is_numeric() && !matches!(c, '\'' | '.' | '_' | '-') {
                return Err(InvalidCommodityError::InvalidCharacter);
            }
        }
        Ok(())
    }
}

impl TryFrom<String> for Commodity<'static> {
    type Error = InvalidCommodityError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Commodity::new(value)
    }
}

impl<'a> TryFrom<&'a str> for Commodity<'a> {
    type Error = InvalidCommodityError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Commodity::new(value)
    }
}

impl<'a> AsRef<str> for Commodity<'a> {
    fn as_ref(&self) -> &str {
        &self.commodity
    }
}

/// Macro to create a commodity
///
/// # Example
/// ```
/// use beancount_rs::model::commodity;
///
/// let com = commodity!(USD);
/// ```
#[macro_export]
macro_rules! commodity_ {
    ($name:ident) => {
        $crate::model::Commodity::new(stringify!($name)).unwrap()
    };
}
pub use commodity_ as commodity;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commodity_macro() {
        let usd = commodity!(USD);
        assert_eq!(usd.as_ref(), "USD");

        let btc = commodity!(BTC);
        assert_eq!(btc.as_ref(), "BTC");

        let eur = commodity!(EUR);
        assert_eq!(eur.as_ref(), "EUR");
    }

    #[test]
    fn test_valid_commodities() {
        assert!(Commodity::new("USD").is_ok());
        assert!(Commodity::new("EUR").is_ok());
        assert!(Commodity::new("GBP").is_ok());
        assert!(Commodity::new("BTC").is_ok());
        assert!(Commodity::new("ETH").is_ok());
        assert!(Commodity::new("SPY").is_ok());
        assert!(Commodity::new("VTI").is_ok());
        assert!(Commodity::new("AAPL").is_ok());
        assert!(Commodity::new("GOOGL").is_ok());
        assert!(Commodity::new("A").is_ok()); // Single character
        assert!(Commodity::new("A1").is_ok()); // Ends with number
        assert!(Commodity::new("AB1").is_ok()); // Ends with number
        assert!(Commodity::new("A'B").is_ok()); // Contains apostrophe
        assert!(Commodity::new("A.B").is_ok()); // Contains period
        assert!(Commodity::new("A_B").is_ok()); // Contains underscore
        assert!(Commodity::new("A-B").is_ok()); // Contains dash
        assert!(Commodity::new("A'B.C_D-E1").is_ok()); // Multiple punctuation
        assert!(Commodity::new("ABCDEFGHIJKLMNOPQR123456").is_ok()); // Max length (24 chars)
    }

    #[test]
    fn test_empty_commodity() {
        let result = Commodity::new("");
        assert_eq!(result.unwrap_err(), InvalidCommodityError::Empty);
    }

    #[test]
    fn test_invalid_start_lowercase() {
        let result = Commodity::new("usd");
        assert_eq!(result.unwrap_err(), InvalidCommodityError::InvalidStart);
    }

    #[test]
    fn test_invalid_start_number() {
        let result = Commodity::new("1USD");
        assert_eq!(result.unwrap_err(), InvalidCommodityError::InvalidStart);
    }

    #[test]
    fn test_invalid_start_punctuation() {
        let result = Commodity::new("-USD");
        assert_eq!(result.unwrap_err(), InvalidCommodityError::InvalidStart);

        let result = Commodity::new("_USD");
        assert_eq!(result.unwrap_err(), InvalidCommodityError::InvalidStart);

        let result = Commodity::new(".USD");
        assert_eq!(result.unwrap_err(), InvalidCommodityError::InvalidStart);

        let result = Commodity::new("'USD");
        assert_eq!(result.unwrap_err(), InvalidCommodityError::InvalidStart);
    }

    #[test]
    fn test_invalid_end_lowercase() {
        let result = Commodity::new("USd");
        assert_eq!(result.unwrap_err(), InvalidCommodityError::InvalidEnd);
    }

    #[test]
    fn test_invalid_end_punctuation() {
        let result = Commodity::new("USD-");
        assert_eq!(result.unwrap_err(), InvalidCommodityError::InvalidEnd);

        let result = Commodity::new("USD_");
        assert_eq!(result.unwrap_err(), InvalidCommodityError::InvalidEnd);

        let result = Commodity::new("USD.");
        assert_eq!(result.unwrap_err(), InvalidCommodityError::InvalidEnd);

        let result = Commodity::new("USD'");
        assert_eq!(result.unwrap_err(), InvalidCommodityError::InvalidEnd);
    }

    #[test]
    fn test_invalid_characters() {
        let result = Commodity::new("US@D");
        assert_eq!(result.unwrap_err(), InvalidCommodityError::InvalidCharacter);

        let result = Commodity::new("US#D");
        assert_eq!(result.unwrap_err(), InvalidCommodityError::InvalidCharacter);

        let result = Commodity::new("US$D");
        assert_eq!(result.unwrap_err(), InvalidCommodityError::InvalidCharacter);

        let result = Commodity::new("US%D");
        assert_eq!(result.unwrap_err(), InvalidCommodityError::InvalidCharacter);

        let result = Commodity::new("US D");
        assert_eq!(result.unwrap_err(), InvalidCommodityError::InvalidCharacter);

        let result = Commodity::new("US!D");
        assert_eq!(result.unwrap_err(), InvalidCommodityError::InvalidCharacter);

        let result = Commodity::new("USd1"); // lowercase in middle
        assert_eq!(result.unwrap_err(), InvalidCommodityError::InvalidCharacter);
    }

    #[test]
    fn test_too_long() {
        let long_commodity = "ABCDEFGHIJKLMNOPQRSTUVWXY"; // 25 characters
        let result = Commodity::new(long_commodity);
        assert_eq!(result.unwrap_err(), InvalidCommodityError::TooLong);

        let very_long_commodity = "ABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJ"; // Much longer
        let result = Commodity::new(very_long_commodity);
        assert_eq!(result.unwrap_err(), InvalidCommodityError::TooLong);
    }

    #[test]
    fn test_try_from_string() {
        let commodity: Result<Commodity, _> = "USD".to_string().try_into();
        assert!(commodity.is_ok());

        let commodity: Result<Commodity, _> = "invalid".to_string().try_into();
        assert!(commodity.is_err());
    }

    #[test]
    fn test_try_from_str() {
        let commodity: Result<Commodity, _> = "USD".try_into();
        assert!(commodity.is_ok());

        let commodity: Result<Commodity, _> = "invalid".try_into();
        assert!(commodity.is_err());
    }

    #[test]
    fn test_as_ref() {
        let commodity = Commodity::new("USD").unwrap();
        assert_eq!(commodity.as_ref(), "USD");
    }

    #[test]
    fn test_display() {
        let commodity = Commodity::new("BTC").unwrap();
        assert_eq!(format!("{}", commodity), "BTC");
    }

    #[test]
    fn test_clone_and_equality() {
        let commodity1 = Commodity::new("USD").unwrap();
        let commodity2 = commodity1.clone();
        assert_eq!(commodity1, commodity2);
    }

    #[test]
    fn test_edge_cases() {
        // Test maximum length boundary
        let max_length_commodity = "A".repeat(24);
        assert!(Commodity::new(&max_length_commodity).is_ok());

        // Test just over maximum length
        let over_max_length = "A".repeat(25);
        assert_eq!(
            Commodity::new(&over_max_length).unwrap_err(),
            InvalidCommodityError::TooLong
        );

        // Test single character with valid end rules
        assert!(Commodity::new("A").is_ok());
        assert!(Commodity::new("Z").is_ok());

        // Test two character combinations
        assert!(Commodity::new("AB").is_ok()); // Letter + Letter
        assert!(Commodity::new("A1").is_ok()); // Letter + Number

        // Invalid two character combinations
        assert_eq!(
            Commodity::new("A-").unwrap_err(),
            InvalidCommodityError::InvalidEnd
        );
    }

    #[test]
    fn test_valid_punctuation_combinations() {
        // Test all valid punctuation characters in middle positions
        assert!(Commodity::new("A'B").is_ok());
        assert!(Commodity::new("A.B").is_ok());
        assert!(Commodity::new("A_B").is_ok());
        assert!(Commodity::new("A-B").is_ok());

        // Test combinations of punctuation
        assert!(Commodity::new("A'B.C").is_ok());
        assert!(Commodity::new("A_B-C").is_ok());
        assert!(Commodity::new("A1'B2.C3_D4-E5").is_ok());
    }
}
