use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CnpjError {
    #[error("Invalid CNPJ length: expected 14 digits")]
    InvalidLength,
    #[error("CNPJ cannot consist of identical digits")]
    IdenticalDigits,
    #[error("Invalid CNPJ checksum")]
    InvalidChecksum,
    #[error("CNPJ must contain only digits or valid separators")]
    InvalidFormat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cnpj(String);

impl Serialize for Cnpj {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Cnpj {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Cnpj::new(&s).map_err(serde::de::Error::custom)
    }
}

impl Cnpj {
    pub fn new(value: &str) -> Result<Self, CnpjError> {
        let cleaned: String = value.chars().filter(|c| c.is_ascii_digit()).collect();

        if cleaned.len() != 14 {
            return Err(CnpjError::InvalidLength);
        }

        if is_all_equal(&cleaned) {
            return Err(CnpjError::IdenticalDigits);
        }

        let digits: Vec<u32> = cleaned.chars().map(|c| c.to_digit(10).unwrap()).collect();

        if !validate_checksum(&digits) {
            return Err(CnpjError::InvalidChecksum);
        }

        Ok(Self(cleaned))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Corrected formatting
impl Cnpj {
    pub fn formatted_correctly(&self) -> String {
        format!(
            "{}.{}.{}/{}-{}",
            &self.0[0..2],
            &self.0[2..5],
            &self.0[5..8],
            &self.0[8..12],
            &self.0[12..14]
        )
    }
}

fn is_all_equal(s: &str) -> bool {
    let first = s.chars().next().unwrap();
    s.chars().all(|c| c == first)
}

fn validate_checksum(digits: &[u32]) -> bool {
    // First digit
    let weights1 = [5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
    let mut sum = 0;
    for (i, &weight) in weights1.iter().enumerate() {
        sum += digits[i] * weight;
    }
    let mut check_digit = sum % 11;
    if check_digit < 2 {
        check_digit = 0;
    } else {
        check_digit = 11 - check_digit;
    }
    if check_digit != digits[12] {
        return false;
    }

    // Second digit
    let weights2 = [6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
    sum = 0;
    for (i, &weight) in weights2.iter().enumerate() {
        sum += digits[i] * weight;
    }
    check_digit = sum % 11;
    if check_digit < 2 {
        check_digit = 0;
    } else {
        check_digit = 11 - check_digit;
    }
    if check_digit != digits[13] {
        return false;
    }

    true
}

impl FromStr for Cnpj {
    type Err = CnpjError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl TryFrom<String> for Cnpj {
    type Error = CnpjError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl fmt::Display for Cnpj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cnpj_validation() {
        let cases = vec![
            ("12.345.678/0001-95", Ok("12345678000195")), // Valid formatted
            ("12345678000195", Ok("12345678000195")),     // Valid unformatted
            ("11111111111111", Err(CnpjError::IdenticalDigits)),
            ("12345678000196", Err(CnpjError::InvalidChecksum)),
            ("1234567800019", Err(CnpjError::InvalidLength)),
            ("123456780001955", Err(CnpjError::InvalidLength)),
            ("ab12.345.678/0001-95", Ok("12345678000195")), // Cleaning
        ];

        for (input, expected) in cases {
            let result = Cnpj::new(input);
            match expected {
                Ok(val) => {
                    assert!(
                        result.is_ok(),
                        "Expected Ok for {}, got {:?}",
                        input,
                        result
                    );
                    assert_eq!(result.unwrap().as_str(), val);
                }
                Err(err) => {
                    assert!(
                        result.is_err(),
                        "Expected Err for {}, got {:?}",
                        input,
                        result
                    );
                    assert_eq!(result.unwrap_err(), err);
                }
            }
        }
    }

    #[test]
    fn test_cnpj_formatting() {
        let cnpj = Cnpj::new("12345678000195").unwrap();
        assert_eq!(cnpj.formatted_correctly(), "12.345.678/0001-95");
    }
}
