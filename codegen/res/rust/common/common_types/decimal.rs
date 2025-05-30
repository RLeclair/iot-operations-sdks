/* This file will be copied into the folder for generated code. */

use std::ops::{Deref, DerefMut};
use std::str::FromStr;

use bigdecimal;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug)]
pub struct Decimal(bigdecimal::BigDecimal);

impl Deref for Decimal {
    type Target = bigdecimal::BigDecimal;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Decimal {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Serialize for Decimal {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(self.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for Decimal {
    fn deserialize<D>(deserializer: D) -> Result<Decimal, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        Ok(Decimal(bigdecimal::BigDecimal::from_str(&s).map_err(de::Error::custom)?))
    }
}
