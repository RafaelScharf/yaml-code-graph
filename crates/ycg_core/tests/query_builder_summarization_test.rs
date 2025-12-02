// Integration test for QueryBuilder signature summarization
// Validates Requirements 5.1, 5.2, 5.3, 5.4, 5.6

use ycg_core::model::{ScipSymbolKind, SymbolNode};
use ycg_core::signature_extractor::SignatureExtractor;

#[test]
fn test_query_builder_summarization_integration() {
    // Simulate a variable with a long QueryBuilder signature
    let long_qb_signature = "const activeUsers = await this.userRepository.createQueryBuilder('User', 'u').select('u.id, u.name, u.email, u.createdAt, u.updatedAt').where('u.active = :active', { active: true }).andWhere('u.deletedAt IS NULL').orderBy('u.createdAt', 'DESC').limit(100).getMany()";

    let node = SymbolNode {
        id: "test_qb_var".to_string(),
        name: "activeUsers".to_string(),
        kind: ScipSymbolKind::Variable,
        parent_id: None,
        documentation: None,
        signature: Some(long_qb_signature.to_string()),
        logic: None,
    };

    // Extract signature
    let result = SignatureExtractor::extract_signature(&node);

    assert!(result.is_some(), "Should extract signature");

    let signature = result.unwrap();

    // Validate Requirements 5.4, 5.6: Summarized to < 100 chars
    assert!(
        signature.len() < 100,
        "Signature should be < 100 chars, got: {} (len: {})",
        signature,
        signature.len()
    );

    // Validate format: variableName: EntityType[]
    assert_eq!(signature, "activeUsers: User[]");

    // Validate it's much shorter than original
    assert!(
        signature.len() < long_qb_signature.len() / 3,
        "Summarized signature should be significantly shorter"
    );
}

#[test]
fn test_query_builder_get_one_summarization() {
    let qb_signature = "const user = await this.userRepository.createQueryBuilder('User').select('user.id, user.name').where('user.id = :id', { id: userId }).getOne()";

    let node = SymbolNode {
        id: "test_qb_single".to_string(),
        name: "user".to_string(),
        kind: ScipSymbolKind::Variable,
        parent_id: None,
        documentation: None,
        signature: Some(qb_signature.to_string()),
        logic: None,
    };

    let result = SignatureExtractor::extract_signature(&node);
    assert!(result.is_some());

    let signature = result.unwrap();

    // Should not have array notation for getOne
    assert_eq!(signature, "user: User");
    assert!(!signature.contains("[]"));
}

#[test]
fn test_query_builder_with_joins() {
    let qb_signature = "const usersWithProfiles = await this.userRepository.createQueryBuilder('User', 'u').leftJoin('u.profile', 'profile').leftJoin('u.posts', 'posts').select('u.id, u.name, profile.bio, posts.title').where('u.active = true').getMany()";

    let node = SymbolNode {
        id: "test_qb_joins".to_string(),
        name: "usersWithProfiles".to_string(),
        kind: ScipSymbolKind::Variable,
        parent_id: None,
        documentation: None,
        signature: Some(qb_signature.to_string()),
        logic: None,
    };

    let result = SignatureExtractor::extract_signature(&node);
    assert!(result.is_some());

    let signature = result.unwrap();

    // Should be summarized despite having joins
    assert_eq!(signature, "usersWithProfiles: User[]");
    assert!(signature.len() < 100);
}

#[test]
fn test_query_builder_fallback_to_query_result() {
    // QueryBuilder without clear entity type (empty createQueryBuilder)
    let qb_signature = "const results = await someRepository.createQueryBuilder().select('field1').select('field2').where('condition = :value').getMany()";

    let node = SymbolNode {
        id: "test_qb_fallback".to_string(),
        name: "results".to_string(),
        kind: ScipSymbolKind::Variable,
        parent_id: None,
        documentation: None,
        signature: Some(qb_signature.to_string()),
        logic: None,
    };

    let result = SignatureExtractor::extract_signature(&node);
    assert!(result.is_some());

    let signature = result.unwrap();

    // Should fallback to QueryResult when entity type cannot be inferred
    assert_eq!(signature, "results: QueryResult[]");
}

#[test]
fn test_non_query_builder_not_summarized() {
    // Regular method signature should not be summarized
    let regular_signature =
        "async function fetchUserData(userId: string, includeProfile: boolean): Promise<UserData>";

    let node = SymbolNode {
        id: "test_regular".to_string(),
        name: "fetchUserData".to_string(),
        kind: ScipSymbolKind::Method,
        parent_id: None,
        documentation: None,
        signature: Some(regular_signature.to_string()),
        logic: None,
    };

    let result = SignatureExtractor::extract_signature(&node);
    assert!(result.is_some());

    let signature = result.unwrap();

    // Should be compacted normally, not summarized as QueryBuilder
    assert!(signature.contains("fetchUserData"));
    assert!(signature.contains("userId:str"));
    assert!(signature.contains("includeProfile:bool"));
}

#[test]
fn test_short_query_builder_not_summarized() {
    // Short QueryBuilder (< 100 chars) should not trigger summarization
    let short_qb = "createQueryBuilder('User').getMany()";

    let node = SymbolNode {
        id: "test_short_qb".to_string(),
        name: "users".to_string(),
        kind: ScipSymbolKind::Variable,
        parent_id: None,
        documentation: None,
        signature: Some(short_qb.to_string()),
        logic: None,
    };

    let result = SignatureExtractor::extract_signature(&node);
    assert!(result.is_some());

    let signature = result.unwrap();

    // Should not be summarized (doesn't meet length requirement)
    // Will be parsed as a regular signature
    assert_eq!(signature, "createQueryBuilder()");
}

#[test]
fn test_query_builder_with_complex_entity_name() {
    let qb_signature = "const entities = await this.repository.createQueryBuilder('UserProfileEntity', 'upe').select('upe.id').where('upe.verified = true').getMany()";

    let node = SymbolNode {
        id: "test_complex_entity".to_string(),
        name: "entities".to_string(),
        kind: ScipSymbolKind::Variable,
        parent_id: None,
        documentation: None,
        signature: Some(qb_signature.to_string()),
        logic: None,
    };

    let result = SignatureExtractor::extract_signature(&node);
    assert!(result.is_some());

    let signature = result.unwrap();

    // Should extract the full entity name
    assert_eq!(signature, "entities: UserProfileEntity[]");
}
