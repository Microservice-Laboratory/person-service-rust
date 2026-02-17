use std::fmt;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CpfError {
    #[error("Invalid CPF length: expected 11 digits")]
    InvalidLength,
    #[error("CPF cannot consist of identical digits")]
    IdenticalDigits,
    #[error("Invalid CPF checksum")]
    InvalidChecksum,
    #[error("CPF must contain only digits or valid separators")]
    InvalidFormat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cpf(String);

impl Serialize for Cpf {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Cpf {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Cpf::new(&s).map_err(serde::de::Error::custom)
    }
}

impl Cpf {
    pub fn new(value: &str) -> Result<Self, CpfError> {
        if value.chars().any(|c| !c.is_ascii_digit() && !['.', '-', '/'].contains(&c)) {
            return Err(CpfError::InvalidFormat);
        }
        
        let cleaned: String = value.chars().filter(|c| c.is_ascii_digit()).collect();

        if cleaned.len() != 11 {
            return Err(CpfError::InvalidLength);
        }

        if is_all_equal(&cleaned) {
            return Err(CpfError::IdenticalDigits);
        }

        let digits: Vec<u32> = cleaned
            .chars()
            .map(|c| c.to_digit(10).unwrap())
            .collect();

        if !validate_checksum(&digits) {
            return Err(CpfError::InvalidChecksum);
        }

        Ok(Self(cleaned))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn formatted(&self) -> String {
        format!(
            "{}.{}.{}-{}",
            &self.0[0..3],
            &self.0[3..6],
            &self.0[6..9],
            &self.0[9..11]
        )
    }
}

fn is_all_equal(s: &str) -> bool {
    let first = s.chars().next().unwrap();
    s.chars().all(|c| c == first)
}

fn validate_checksum(digits: &[u32]) -> bool {
    // First digit
    let mut sum = 0;
    for (i, digit) in digits[0..9].iter().enumerate() {
        sum += digit * (10 - i as u32);
    }
    let mut check_digit = (sum * 10) % 11;
    if check_digit == 10 {
        check_digit = 0;
    }
    if check_digit != digits[9] {
        return false;
    }

    // Second digit
    sum = 0;
    for (i, digit) in digits[0..10].iter().enumerate() {
        sum += digit * (11 - i as u32);
    }
    check_digit = (sum * 10) % 11;
    if check_digit == 10 {
        check_digit = 0;
    }
    if check_digit != digits[10] {
        return false;
    }

    true
}

impl FromStr for Cpf {
    type Err = CpfError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl TryFrom<String> for Cpf {
    type Error = CpfError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl fmt::Display for Cpf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpf_validation() {
        let cases = vec![
            ("52998224725", Ok("52998224725")),     // Valid unformatted
            ("529.982.247-25", Ok("52998224725")),   // Valid formatted
            ("12345678909", Ok("12345678909")),     // Valid unformatted
            ("123.456.789-09", Ok("12345678909")),   // Valid formatted
            ("11111111111", Err(CpfError::IdenticalDigits)),
            ("222.222.222-22", Err(CpfError::IdenticalDigits)),
            ("1234567890", Err(CpfError::InvalidLength)),
            ("123456789012", Err(CpfError::InvalidLength)),
            ("12345678900", Err(CpfError::InvalidChecksum)),
            ("abc12345678909", Ok("12345678909")), // Cleaning should work
        ];

        for (input, expected) in cases {
            let result = Cpf::new(input);
            match expected {
                Ok(val) => {
                    assert!(result.is_ok(), "Expected Ok for {}, got {:?}", input, result);
                    assert_eq!(result.unwrap().as_str(), val);
                }
                Err(err) => {
                    assert!(result.is_err(), "Expected Err for {}, got {:?}", input, result);
                    assert_eq!(result.unwrap_err(), err);
                }
            }
        }
    }

    #[test]
    fn test_cpf_formatting() {
        let cpf = Cpf::new("52998224725").unwrap();
        assert_eq!(cpf.formatted(), "529.982.247-25");
    }
}
