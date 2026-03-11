use std::fmt;

/// A SHA-256 hash value (32 bytes).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hash([u8; 32]);

impl Hash {
    /// Create a `Hash` from a raw 32-byte array.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Return the inner bytes by reference.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Consume the wrapper and return the inner bytes.
    #[must_use]
    pub const fn into_bytes(self) -> [u8; 32] {
        self.0
    }
}

impl From<[u8; 32]> for Hash {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl AsRef<[u8; 32]> for Hash {
    fn as_ref(&self) -> &[u8; 32] {
        &self.0
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash(")?;
        for byte in &self.0 {
            write!(f, "{byte:02x}")?;
        }
        write!(f, ")")
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_is_lowercase_hex() {
        let h = Hash::from_bytes([0xab; 32]);
        let s = h.to_string();
        assert_eq!(s.len(), 64);
        assert!(s.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(&s[..4], "abab");
    }

    #[test]
    fn debug_wraps_hex() {
        let h = Hash::from_bytes([0x00; 32]);
        let dbg = format!("{h:?}");
        assert!(dbg.starts_with("Hash("));
        assert!(dbg.ends_with(')'));
    }

    #[test]
    fn roundtrip_bytes() {
        let raw = [42u8; 32];
        let h = Hash::from_bytes(raw);
        assert_eq!(h.into_bytes(), raw);
        assert_eq!(*h.as_bytes(), raw);
    }

    #[test]
    fn equality() {
        let a = Hash::from_bytes([1u8; 32]);
        let b = Hash::from_bytes([1u8; 32]);
        let c = Hash::from_bytes([2u8; 32]);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
