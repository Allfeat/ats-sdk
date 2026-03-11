//! Deterministic, language-agnostic byte serialization for ATS data.
//!
//! Encoding rules (protocol-fixed):
//! - String: `[length as u32 LE][UTF-8 bytes]`
//! - Optional string: `0x00` for None, `0x01 + string encoding` for Some
//! - Roles: deduplicated, sorted ascending by tag, `[count as u8][tag bytes…]`
//! - Creator field order: `full_name`, `email`, `roles`, `ipi`, `isni`

use crate::model::{Creator, Role};

/// Canonical byte encoding of a creator (for leaf hashing).
#[must_use]
pub fn canonical_encode_creator(creator: &Creator) -> Vec<u8> {
    let mut buf = Vec::new();
    encode_string(&mut buf, &creator.full_name);
    encode_string(&mut buf, &creator.email);
    encode_roles(&mut buf, &creator.roles);
    encode_optional_string(&mut buf, creator.ipi.as_ref());
    encode_optional_string(&mut buf, creator.isni.as_ref());
    buf
}

/// Canonical byte encoding of the title string (for commitment preimage).
#[must_use]
pub fn canonical_encode_title(title: &str) -> Vec<u8> {
    let mut buf = Vec::new();
    encode_string(&mut buf, title);
    buf
}

// ----- Internal helpers -----

fn encode_string(buf: &mut Vec<u8>, s: &str) {
    // Protocol uses u32 LE for string lengths. Strings > 4 GiB are unreachable
    // in practice (validated upstream), so truncation cannot occur.
    #[allow(clippy::cast_possible_truncation)]
    let len = s.len() as u32;
    buf.extend_from_slice(&len.to_le_bytes());
    buf.extend_from_slice(s.as_bytes());
}

fn encode_optional_string(buf: &mut Vec<u8>, opt: Option<&String>) {
    match opt {
        None => buf.push(0x00),
        Some(s) => {
            buf.push(0x01);
            encode_string(buf, s);
        }
    }
}

fn encode_roles(buf: &mut Vec<u8>, roles: &[Role]) {
    let mut tags: Vec<u8> = roles.iter().map(|r| *r as u8).collect();
    tags.sort_unstable();
    tags.dedup();
    debug_assert!(!tags.is_empty(), "roles must not be empty (validate first)");
    debug_assert!(tags.len() <= 4, "at most 4 distinct roles");
    // At most 4 roles, so len always fits in u8.
    #[allow(clippy::cast_possible_truncation)]
    let count = tags.len() as u8;
    buf.push(count);
    buf.extend_from_slice(&tags);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Role;

    #[test]
    fn encode_string_basic() {
        let mut buf = Vec::new();
        encode_string(&mut buf, "Alice");
        // length 5 as u32 LE = [05, 00, 00, 00], then "Alice" UTF-8
        assert_eq!(buf, [0x05, 0x00, 0x00, 0x00, 0x41, 0x6C, 0x69, 0x63, 0x65]);
    }

    #[test]
    fn encode_string_empty() {
        let mut buf = Vec::new();
        encode_string(&mut buf, "");
        assert_eq!(buf, [0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn encode_optional_none() {
        let mut buf = Vec::new();
        encode_optional_string(&mut buf, None);
        assert_eq!(buf, [0x00]);
    }

    #[test]
    fn encode_optional_some() {
        let mut buf = Vec::new();
        let val = "AB".to_string();
        encode_optional_string(&mut buf, Some(&val));
        // 0x01 + length(2) as u32 LE + "AB"
        assert_eq!(buf, [0x01, 0x02, 0x00, 0x00, 0x00, 0x41, 0x42]);
    }

    #[test]
    fn encode_roles_sorted_and_deduped() {
        let mut buf = Vec::new();
        // Provide in reverse order with a duplicate
        encode_roles(
            &mut buf,
            &[Role::Adapter, Role::Author, Role::Adapter, Role::Composer],
        );
        // Sorted: Author(0), Composer(1), Adapter(3) — deduped
        assert_eq!(buf, [0x03, 0x00, 0x01, 0x03]);
    }

    #[test]
    fn encode_roles_single() {
        let mut buf = Vec::new();
        encode_roles(&mut buf, &[Role::Arranger]);
        assert_eq!(buf, [0x01, 0x02]);
    }

    #[test]
    fn canonical_creator_full() {
        let creator = Creator {
            full_name: "Alice".into(),
            email: "a@b.c".into(),
            roles: vec![Role::Composer, Role::Author],
            ipi: Some("12345".into()),
            isni: None,
        };
        let bytes = canonical_encode_creator(&creator);
        // Verify field order: full_name, email, roles, ipi, isni
        let mut expected = Vec::new();
        // full_name "Alice"
        expected.extend_from_slice(&5u32.to_le_bytes());
        expected.extend_from_slice(b"Alice");
        // email "a@b.c"
        expected.extend_from_slice(&5u32.to_le_bytes());
        expected.extend_from_slice(b"a@b.c");
        // roles [Composer(1), Author(0)] -> sorted [0, 1]
        expected.extend_from_slice(&[0x02, 0x00, 0x01]);
        // ipi Some("12345")
        expected.push(0x01);
        expected.extend_from_slice(&5u32.to_le_bytes());
        expected.extend_from_slice(b"12345");
        // isni None
        expected.push(0x00);

        assert_eq!(bytes, expected);
    }

    #[test]
    fn canonical_title() {
        let bytes = canonical_encode_title("My Song");
        let mut expected = Vec::new();
        expected.extend_from_slice(&7u32.to_le_bytes());
        expected.extend_from_slice(b"My Song");
        assert_eq!(bytes, expected);
    }

    #[test]
    fn canonical_encoding_is_deterministic() {
        let creator = Creator {
            full_name: "Bob".into(),
            email: "bob@test.com".into(),
            roles: vec![Role::Author],
            ipi: None,
            isni: None,
        };
        let a = canonical_encode_creator(&creator);
        let b = canonical_encode_creator(&creator);
        assert_eq!(a, b);
    }
}
