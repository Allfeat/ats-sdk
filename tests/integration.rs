//! End-to-end integration tests for the ATS SDK.
//!
//! These tests exercise the full pipeline: input -> commitment -> verification -> Merkle proofs.
//! They also provide deterministic test vectors for cross-language validation.

use ats_sdk::{
    generate_commitment, generate_creator_proof, hash_media, verify_commitment,
    verify_creator_inclusion, AtsError, AtsInput, Creator, Hash, Role, MERKLE_DEPTH,
    PROTOCOL_VERSION,
};

// ===== Helpers =====

fn alice() -> Creator {
    Creator {
        full_name: "Alice Dupont".into(),
        email: "alice@music.com".into(),
        roles: vec![Role::Author, Role::Composer],
        ipi: Some("00012345678".into()),
        isni: None,
    }
}

fn bob() -> Creator {
    Creator {
        full_name: "Bob Martin".into(),
        email: "bob@studio.fr".into(),
        roles: vec![Role::Arranger],
        ipi: None,
        isni: Some("0000000121032683".into()),
    }
}

fn charlie() -> Creator {
    Creator {
        full_name: "Charlie Vega".into(),
        email: "charlie@label.io".into(),
        roles: vec![Role::Adapter, Role::Author],
        ipi: None,
        isni: None,
    }
}

fn standard_input() -> AtsInput {
    AtsInput {
        title: "Ma Chanson d'Amour".into(),
        creators: vec![alice(), bob(), charlie()],
    }
}

const MEDIA_BYTES: &[u8] = b"fake audio content for testing purposes";

// ===== Full pipeline tests =====

#[test]
fn full_pipeline_generate_and_verify() {
    let input = standard_input();
    let proof = generate_commitment(&input, MEDIA_BYTES).unwrap();

    assert_eq!(proof.on_chain.protocol_version, PROTOCOL_VERSION);

    let standalone_media_hash = hash_media(MEDIA_BYTES).unwrap();
    assert_eq!(proof.media_hash, standalone_media_hash);

    assert!(verify_commitment(&input, MEDIA_BYTES, &proof.on_chain).unwrap());

    assert_eq!(proof.creator_leaves().len(), 32);
}

#[test]
fn commitment_is_deterministic() {
    let input = standard_input();
    let a = generate_commitment(&input, MEDIA_BYTES).unwrap();
    let b = generate_commitment(&input, MEDIA_BYTES).unwrap();
    assert_eq!(a.on_chain, b.on_chain);
    assert_eq!(a.media_hash, b.media_hash);
    assert_eq!(a.creators_merkle_root(), b.creators_merkle_root());
    assert_eq!(a.creator_leaves(), b.creator_leaves());
}

#[test]
fn any_change_invalidates_commitment() {
    let input = standard_input();
    let proof = generate_commitment(&input, MEDIA_BYTES).unwrap();

    // Change title
    let mut changed = standard_input();
    changed.title = "Different Title".into();
    assert!(!verify_commitment(&changed, MEDIA_BYTES, &proof.on_chain).unwrap());

    // Change media
    assert!(!verify_commitment(&input, b"different media", &proof.on_chain).unwrap());

    // Change a creator's name
    let mut changed = standard_input();
    changed.creators[0].full_name = "Alice Changed".into();
    assert!(!verify_commitment(&changed, MEDIA_BYTES, &proof.on_chain).unwrap());

    // Change a creator's role
    let mut changed = standard_input();
    changed.creators[1].roles = vec![Role::Adapter];
    assert!(!verify_commitment(&changed, MEDIA_BYTES, &proof.on_chain).unwrap());

    // Add a creator
    let mut changed = standard_input();
    changed.creators.push(Creator {
        full_name: "Diana".into(),
        email: "diana@test.com".into(),
        roles: vec![Role::Composer],
        ipi: None,
        isni: None,
    });
    assert!(!verify_commitment(&changed, MEDIA_BYTES, &proof.on_chain).unwrap());

    // Remove a creator
    let mut changed = standard_input();
    changed.creators.pop();
    assert!(!verify_commitment(&changed, MEDIA_BYTES, &proof.on_chain).unwrap());
}

// ===== Selective disclosure (Merkle proofs) =====

#[test]
fn creator_selective_disclosure_all_creators() {
    let input = standard_input();
    let proof = generate_commitment(&input, MEDIA_BYTES).unwrap();

    for i in 0..input.creators.len() {
        let mproof = generate_creator_proof(&input, i).unwrap();
        assert_eq!(mproof.len(), MERKLE_DEPTH);
        assert!(
            verify_creator_inclusion(&input.creators[i], &mproof, &proof.creators_merkle_root()),
            "creator {i} failed verification"
        );
    }
}

#[test]
fn creator_proof_rejects_modified_data() {
    let input = standard_input();
    let proof = generate_commitment(&input, MEDIA_BYTES).unwrap();
    let mproof = generate_creator_proof(&input, 0).unwrap();

    let mut fake = alice();
    fake.email = "fake@hacker.com".into();
    assert!(!verify_creator_inclusion(
        &fake,
        &mproof,
        &proof.creators_merkle_root(),
    ));
}

#[test]
fn creator_proof_rejects_wrong_root() {
    let input = standard_input();
    let mproof = generate_creator_proof(&input, 0).unwrap();
    let wrong_root = Hash::from_bytes([0xFF; 32]);
    assert!(!verify_creator_inclusion(
        &input.creators[0],
        &mproof,
        &wrong_root,
    ));
}

#[test]
fn creator_proof_from_stored_tree() {
    let input = standard_input();
    let proof = generate_commitment(&input, MEDIA_BYTES).unwrap();

    for i in 0..input.creators.len() {
        let mproof = proof.creator_proof(i);
        assert!(verify_creator_inclusion(
            &input.creators[i],
            &mproof,
            &proof.creators_merkle_root(),
        ));
    }
}

// ===== Single creator =====

#[test]
fn single_creator_works() {
    let input = AtsInput {
        title: "Solo".into(),
        creators: vec![alice()],
    };
    let proof = generate_commitment(&input, b"solo media").unwrap();
    assert!(verify_commitment(&input, b"solo media", &proof.on_chain).unwrap());

    let mproof = generate_creator_proof(&input, 0).unwrap();
    assert!(verify_creator_inclusion(
        &input.creators[0],
        &mproof,
        &proof.creators_merkle_root(),
    ));
}

// ===== Max creators =====

#[test]
fn max_32_creators() {
    let creators: Vec<Creator> = (0..32)
        .map(|i| Creator {
            full_name: format!("Creator {i}"),
            email: format!("c{i}@test.com"),
            roles: vec![Role::Author],
            ipi: None,
            isni: None,
        })
        .collect();
    let input = AtsInput {
        title: "Collaboration".into(),
        creators,
    };
    let proof = generate_commitment(&input, b"collab media").unwrap();
    assert!(verify_commitment(&input, b"collab media", &proof.on_chain).unwrap());

    for i in 0..32 {
        let mproof = generate_creator_proof(&input, i).unwrap();
        assert!(
            verify_creator_inclusion(&input.creators[i], &mproof, &proof.creators_merkle_root()),
            "creator {i} failed"
        );
    }
}

// ===== Validation edge cases =====

#[test]
fn empty_media_rejected() {
    let input = standard_input();
    let result = generate_commitment(&input, &[]);
    assert!(matches!(result, Err(AtsError::EmptyMedia)));
}

#[test]
fn empty_title_rejected() {
    let input = AtsInput {
        title: String::new(),
        creators: vec![alice()],
    };
    let result = generate_commitment(&input, b"test");
    assert!(matches!(result, Err(AtsError::EmptyTitle)));
}

#[test]
fn creator_index_out_of_bounds() {
    let input = standard_input();
    let result = generate_creator_proof(&input, 100);
    assert!(matches!(
        result,
        Err(AtsError::CreatorIndexOutOfBounds { .. })
    ));
}

// ===== Cross-language test vectors =====
//
// These tests produce deterministic outputs that other implementations
// (TypeScript, etc.) MUST reproduce exactly.

#[test]
fn test_vector_media_hash() {
    let media_hash = hash_media(b"Hello, Allfeat!").unwrap();
    let hex = media_hash.to_string();
    assert_eq!(hex.len(), 64);
    println!("TEST VECTOR \u{2014} media_hash: {hex}");
}

#[test]
fn test_vector_single_creator_commitment() {
    let input = AtsInput {
        title: "Test Vector Song".into(),
        creators: vec![Creator {
            full_name: "Test User".into(),
            email: "test@example.com".into(),
            roles: vec![Role::Author],
            ipi: None,
            isni: None,
        }],
    };
    let proof = generate_commitment(&input, b"test vector media").unwrap();

    println!(
        "TEST VECTOR \u{2014} commitment: {}",
        proof.on_chain.commitment
    );
    println!("TEST VECTOR \u{2014} media_hash: {}", proof.media_hash);
    println!(
        "TEST VECTOR \u{2014} merkle_root: {}",
        proof.creators_merkle_root()
    );
    println!(
        "TEST VECTOR \u{2014} protocol_version: {}",
        proof.on_chain.protocol_version
    );

    assert_eq!(proof.on_chain.protocol_version, 1);
    assert_ne!(proof.on_chain.commitment, Hash::from_bytes([0; 32]));
}

#[test]
fn test_vector_multi_creator_with_all_fields() {
    let input = AtsInput {
        title: "Cross-Language Vector".into(),
        creators: vec![
            Creator {
                full_name: "Alpha".into(),
                email: "alpha@test.com".into(),
                roles: vec![Role::Author, Role::Composer],
                ipi: Some("12345678901".into()),
                isni: Some("000000012103268X".into()),
            },
            Creator {
                full_name: "Beta".into(),
                email: "beta@test.com".into(),
                roles: vec![Role::Arranger, Role::Adapter],
                ipi: None,
                isni: None,
            },
        ],
    };
    let proof = generate_commitment(&input, b"vector media bytes").unwrap();

    println!(
        "TEST VECTOR (multi) \u{2014} commitment: {}",
        proof.on_chain.commitment
    );
    println!(
        "TEST VECTOR (multi) \u{2014} media_hash: {}",
        proof.media_hash
    );
    println!(
        "TEST VECTOR (multi) \u{2014} merkle_root: {}",
        proof.creators_merkle_root()
    );

    assert!(verify_commitment(&input, b"vector media bytes", &proof.on_chain).unwrap());

    for i in 0..2 {
        let mproof = generate_creator_proof(&input, i).unwrap();
        assert!(verify_creator_inclusion(
            &input.creators[i],
            &mproof,
            &proof.creators_merkle_root(),
        ));
    }
}

// ===== Unicode and special characters =====

#[test]
fn unicode_title_and_names() {
    let input = AtsInput {
        title: "La Vie en Rose \u{1f339}".into(),
        creators: vec![Creator {
            full_name: "\u{00c9}dith Piaf".into(),
            email: "edith@chanson.fr".into(),
            roles: vec![Role::Author, Role::Composer],
            ipi: None,
            isni: None,
        }],
    };
    let proof = generate_commitment(&input, b"rose media").unwrap();
    assert!(verify_commitment(&input, b"rose media", &proof.on_chain).unwrap());
}

// ===== Role ordering doesn't affect commitment =====

#[test]
fn role_order_is_canonical() {
    let mut input_a = standard_input();
    input_a.creators[0].roles = vec![Role::Composer, Role::Author];

    let mut input_b = standard_input();
    input_b.creators[0].roles = vec![Role::Author, Role::Composer];

    let a = generate_commitment(&input_a, MEDIA_BYTES).unwrap();
    let b = generate_commitment(&input_b, MEDIA_BYTES).unwrap();
    assert_eq!(a.on_chain.commitment, b.on_chain.commitment);
}

// ===== Duplicate roles are ignored =====

#[test]
fn duplicate_roles_ignored() {
    let mut input_a = standard_input();
    input_a.creators[0].roles = vec![Role::Author];

    let mut input_b = standard_input();
    input_b.creators[0].roles = vec![Role::Author, Role::Author, Role::Author];

    let a = generate_commitment(&input_a, MEDIA_BYTES).unwrap();
    let b = generate_commitment(&input_b, MEDIA_BYTES).unwrap();
    assert_eq!(a.on_chain.commitment, b.on_chain.commitment);
}
