//! Input validation for ATS data.

use alloc::format;

use crate::error::AtsError;
use crate::model::{AtsInput, Creator, MAX_CREATORS};

/// Validate all fields of an [`AtsInput`].
///
/// This is called automatically by [`generate_commitment`](crate::commitment::generate_commitment)
/// and [`generate_creator_proof`](crate::commitment::generate_creator_proof), but can also be
/// called directly for early validation (e.g., before hashing a large media file).
///
/// # Errors
/// Returns an [`AtsError`] describing the first invalid field found.
pub fn validate_input(input: &AtsInput) -> Result<(), AtsError> {
    if input.title.is_empty() {
        return Err(AtsError::EmptyTitle);
    }
    if input.creators.is_empty() {
        return Err(AtsError::NoCreators);
    }
    if input.creators.len() > MAX_CREATORS {
        return Err(AtsError::TooManyCreators {
            count: input.creators.len(),
        });
    }
    for (i, creator) in input.creators.iter().enumerate() {
        validate_creator(creator, i)?;
    }
    Ok(())
}

fn validate_creator(creator: &Creator, index: usize) -> Result<(), AtsError> {
    if creator.full_name.is_empty() {
        return Err(AtsError::CreatorEmptyName { index });
    }
    if creator.email.is_empty() {
        return Err(AtsError::CreatorEmptyEmail { index });
    }
    if creator.roles.is_empty() {
        return Err(AtsError::CreatorNoRoles { index });
    }
    if let Some(ref ipi) = creator.ipi {
        validate_ipi(ipi, index)?;
    }
    if let Some(ref isni) = creator.isni {
        validate_isni(isni, index)?;
    }
    Ok(())
}

fn validate_ipi(ipi: &str, index: usize) -> Result<(), AtsError> {
    if ipi.is_empty() || ipi.len() > 11 {
        return Err(AtsError::CreatorInvalidIpi {
            index,
            reason: format!("length must be 1\u{2013}11, got {}", ipi.len()),
        });
    }
    if !ipi.bytes().all(|b| b.is_ascii_digit()) {
        return Err(AtsError::CreatorInvalidIpi {
            index,
            reason: "must contain only digits".into(),
        });
    }
    Ok(())
}

fn validate_isni(isni: &str, index: usize) -> Result<(), AtsError> {
    if isni.len() != 16 {
        return Err(AtsError::CreatorInvalidIsni {
            index,
            reason: format!("length must be 16, got {}", isni.len()),
        });
    }
    if !isni.bytes().all(|b| b.is_ascii_digit() || b == b'X') {
        return Err(AtsError::CreatorInvalidIsni {
            index,
            reason: "must contain only digits or 'X'".into(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use alloc::{string::String, vec, vec::Vec};

    use super::*;
    use crate::model::Role;

    fn alice() -> Creator {
        Creator {
            full_name: "Alice Dupont".into(),
            email: "alice@example.com".into(),
            roles: vec![Role::Author, Role::Composer],
            ipi: Some("00012345678".into()),
            isni: None,
        }
    }

    fn bob() -> Creator {
        Creator {
            full_name: "Bob Martin".into(),
            email: "bob@example.com".into(),
            roles: vec![Role::Arranger],
            ipi: None,
            isni: Some("0000000121032683".into()),
        }
    }

    fn sample_input() -> AtsInput {
        AtsInput {
            title: "Ma Chanson".into(),
            creators: vec![alice(), bob()],
        }
    }

    #[test]
    fn valid_input_accepted() {
        assert!(validate_input(&sample_input()).is_ok());
    }

    #[test]
    fn empty_title_rejected() {
        let mut input = sample_input();
        input.title = String::new();
        assert!(matches!(validate_input(&input), Err(AtsError::EmptyTitle)));
    }

    #[test]
    fn no_creators_rejected() {
        let input = AtsInput {
            title: "Title".into(),
            creators: vec![],
        };
        assert!(matches!(validate_input(&input), Err(AtsError::NoCreators)));
    }

    #[test]
    fn too_many_creators_rejected() {
        let creators: Vec<Creator> = (0..33)
            .map(|i| Creator {
                full_name: format!("Creator {i}"),
                email: format!("c{i}@test.com"),
                roles: vec![Role::Author],
                ipi: None,
                isni: None,
            })
            .collect();
        let input = AtsInput {
            title: "Title".into(),
            creators,
        };
        assert!(matches!(
            validate_input(&input),
            Err(AtsError::TooManyCreators { count: 33 })
        ));
    }

    #[test]
    fn creator_empty_name_rejected() {
        let mut input = sample_input();
        input.creators[0].full_name = String::new();
        assert!(matches!(
            validate_input(&input),
            Err(AtsError::CreatorEmptyName { index: 0 })
        ));
    }

    #[test]
    fn creator_empty_email_rejected() {
        let mut input = sample_input();
        input.creators[1].email = String::new();
        assert!(matches!(
            validate_input(&input),
            Err(AtsError::CreatorEmptyEmail { index: 1 })
        ));
    }

    #[test]
    fn creator_no_roles_rejected() {
        let mut input = sample_input();
        input.creators[0].roles.clear();
        assert!(matches!(
            validate_input(&input),
            Err(AtsError::CreatorNoRoles { index: 0 })
        ));
    }

    #[test]
    fn ipi_too_long_rejected() {
        let mut input = sample_input();
        input.creators[0].ipi = Some("123456789012".into());
        assert!(matches!(
            validate_input(&input),
            Err(AtsError::CreatorInvalidIpi { index: 0, .. })
        ));
    }

    #[test]
    fn ipi_non_digit_rejected() {
        let mut input = sample_input();
        input.creators[0].ipi = Some("123abc".into());
        assert!(matches!(
            validate_input(&input),
            Err(AtsError::CreatorInvalidIpi { index: 0, .. })
        ));
    }

    #[test]
    fn isni_wrong_length_rejected() {
        let mut input = sample_input();
        input.creators[1].isni = Some("12345".into());
        assert!(matches!(
            validate_input(&input),
            Err(AtsError::CreatorInvalidIsni { index: 1, .. })
        ));
    }

    #[test]
    fn isni_invalid_char_rejected() {
        let mut input = sample_input();
        input.creators[1].isni = Some("000000012103268Z".into());
        assert!(matches!(
            validate_input(&input),
            Err(AtsError::CreatorInvalidIsni { index: 1, .. })
        ));
    }
}
