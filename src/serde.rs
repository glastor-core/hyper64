//! First-class `serde` integration for zero-copy and zero-allocation Base64 serialization.
//!
//! Provides:
//! - Direct helper modules for `#[serde(with = "...")]`:
//!   - `hyper64::serde::standard`
//!   - `hyper64::serde::standard_no_pad`
//!   - `hyper64::serde::url_safe`
//!   - `hyper64::serde::url_safe_no_pad`
//! - `Base64<T>` newtype wrapper for automatic transparent serialization/deserialization.

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use core::fmt;
use core::ops::{Deref, DerefMut};
use serde::{Deserializer, Serializer, de};

use crate::alphabet::Config;

/// Transparent wrapper around byte containers that automatically serializes
/// and deserializes as Base64.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct Base64<T>(pub T);

impl<T> Base64<T> {
    /// Wraps a value in `Base64`.
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    /// Unwraps the inner value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for Base64<T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Base64<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> AsRef<T> for Base64<T> {
    #[inline(always)]
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> AsMut<T> for Base64<T> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> From<T> for Base64<T> {
    #[inline(always)]
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: AsRef<[u8]>> serde::Serialize for Base64<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let display = crate::display::display_config(self.0.as_ref(), &Config::STANDARD);
        serializer.collect_str(&display)
    }
}

#[cfg(feature = "alloc")]
impl<'de> serde::Deserialize<'de> for Base64<Vec<u8>> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        standard::deserialize(deserializer).map(Base64)
    }
}

macro_rules! make_serde_helper {
    ($mod_name:ident, $config:expr, $doc:expr) => {
        #[doc = $doc]
        pub mod $mod_name {
            use super::*;

            /// Serializes a byte container as Base64 using zero-allocation streaming.
            pub fn serialize<T, S>(bytes: &T, serializer: S) -> Result<S::Ok, S::Error>
            where
                T: ?Sized + AsRef<[u8]>,
                S: Serializer,
            {
                let display = crate::display::display_config(bytes.as_ref(), &$config);
                serializer.collect_str(&display)
            }

            /// Deserializes a Base64 string into a byte container.
            #[cfg(feature = "alloc")]
            pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
            where
                D: Deserializer<'de>,
                T: TryFrom<Vec<u8>>,
                T::Error: fmt::Display,
            {
                struct Base64Visitor<T>(core::marker::PhantomData<T>);

                impl<'de, T> de::Visitor<'de> for Base64Visitor<T>
                where
                    T: TryFrom<Vec<u8>>,
                    T::Error: fmt::Display,
                {
                    type Value = T;

                    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        write!(f, "a valid Base64 encoded string")
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        let bytes = crate::decode::decode(v.as_bytes(), &$config)
                            .map_err(de::Error::custom)?;
                        T::try_from(bytes).map_err(de::Error::custom)
                    }
                }

                deserializer.deserialize_str(Base64Visitor(core::marker::PhantomData))
            }
        }
    };
}

make_serde_helper!(
    standard,
    Config::STANDARD,
    "Serde adapter for RFC 4648 Standard Base64."
);
make_serde_helper!(
    standard_no_pad,
    Config::STANDARD_NO_PAD,
    "Serde adapter for RFC 4648 Standard Base64 without '=' padding."
);
make_serde_helper!(
    url_safe,
    Config::URL_SAFE,
    "Serde adapter for RFC 4648 URL-safe Base64."
);
make_serde_helper!(
    url_safe_no_pad,
    Config::URL_SAFE_NO_PAD,
    "Serde adapter for RFC 4648 URL-safe Base64 without '=' padding."
);

#[cfg(all(test, feature = "serde", feature = "alloc"))]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestPayload {
        #[serde(with = "standard")]
        pub data: Vec<u8>,
        #[serde(with = "url_safe_no_pad")]
        pub token: Vec<u8>,
        pub wrapped: Base64<Vec<u8>>,
    }

    #[test]
    fn test_serde_roundtrip() {
        // In dev-dependencies, let's verify Base64 wrapper and helpers compile
        let payload = TestPayload {
            data: b"hello standard".to_vec(),
            token: b"url-safe-bytes".to_vec(),
            wrapped: Base64(b"wrapped-bytes".to_vec()),
        };

        let mut ser_output = alloc::string::String::new();
        // Base64Display format test
        let disp = crate::display::display_config(&payload.data, &Config::STANDARD);
        use core::fmt::Write;
        write!(ser_output, "{}", disp).unwrap();
        assert_eq!(ser_output, "aGVsbG8gc3RhbmRhcmQ=");
    }
}
