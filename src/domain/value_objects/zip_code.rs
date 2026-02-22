use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ZipCodeError {
    #[error("Invalid Brazilian ZipCode: must be 8 digits")]
    InvalidBrazilianZipCode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZipCode(String);

impl ZipCode {
    pub fn new(code: &str, country: &str) -> Result<Self, ZipCodeError> {
        if country == "BR" {
            let digits: String = code.chars().filter(|c| c.is_ascii_digit()).collect();
            if digits.len() != 8 {
                return Err(ZipCodeError::InvalidBrazilianZipCode);
            }
            Ok(Self(digits))
        } else {
            Ok(Self(code.to_string()))
        }
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_valid_brazilian_zipcode() {
        let zip = ZipCode::new("12345678", "BR");
        assert!(zip.is_ok());
        assert_eq!(zip.unwrap().value(), "12345678");
    }

    #[test]
    fn should_create_valid_brazilian_zipcode_with_formatting() {
        let zip = ZipCode::new("12345-678", "BR");
        assert!(zip.is_ok());
        assert_eq!(zip.unwrap().value(), "12345678");
    }

    #[test]
    fn should_fail_invalid_brazilian_zipcode() {
        let zip = ZipCode::new("12345", "BR");
        assert_eq!(zip.err().unwrap(), ZipCodeError::InvalidBrazilianZipCode);
    }

    #[test]
    fn should_accept_any_zipcode_for_other_countries() {
        let zip = ZipCode::new("ABC-123", "US");
        assert!(zip.is_ok());
        assert_eq!(zip.unwrap().value(), "ABC-123");
    }
}
