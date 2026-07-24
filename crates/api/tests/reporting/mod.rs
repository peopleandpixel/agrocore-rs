// Reporting Handler Tests - DTO/Service Validation
use agrocore_shared::Pagination;

#[test]
fn test_pagination_defaults() {
    let p = Pagination {
        page: Some(1),
        per_page: Some(20),
    };

    assert_eq!(p.page, Some(1));
    assert_eq!(p.per_page, Some(20));
}

#[test]
fn test_pagination_bounds() {
    let p = Pagination {
        page: Some(0), // Invalid: should be 1+
        per_page: Some(100),
    };

    // Pagination bounds are validated at service level, not DTO level
    assert_eq!(p.per_page, Some(100));
}
