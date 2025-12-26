//! Common pagination types.

/// Cursor value to request the first page of results.
///
/// Use this constant instead of the magic string `"*"` when starting pagination.
pub const FIRST_PAGE_CURSOR: &str = "*";

// =============================================================================
// Sort
// =============================================================================

/// Sort direction for ordering results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortDirection {
    /// Ascending order (A-Z, 0-9, oldest first).
    #[default]
    Asc,
    /// Descending order (Z-A, 9-0, newest first).
    Desc,
}

/// Specifies a field and direction for sorting.
#[derive(Debug, Clone)]
pub struct SortField {
    /// The field name to sort by.
    pub field: String,
    /// Sort direction. Defaults to ascending.
    pub direction: SortDirection,
}

impl SortField {
    /// Creates a new sort field with ascending direction.
    pub fn asc(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            direction: SortDirection::Asc,
        }
    }

    /// Creates a new sort field with descending direction.
    pub fn desc(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            direction: SortDirection::Desc,
        }
    }
}

/// Formats a list of sort fields to the string format expected by REST APIs.
///
/// Format: "field:DIRECTION,field2:DIRECTION" (e.g., "created_at:DESC,name:ASC")
///
/// Returns `None` if the list is empty.
pub fn format_sort_fields(sort: &[SortField]) -> Option<String> {
    if sort.is_empty() {
        return None;
    }

    Some(
        sort.iter()
            .map(|s| {
                let dir = match s.direction {
                    SortDirection::Asc => "ASC",
                    SortDirection::Desc => "DESC",
                };
                format!("{}:{}", s.field, dir)
            })
            .collect::<Vec<_>>()
            .join(","),
    )
}

// =============================================================================
// Pagination
// =============================================================================

/// Common pagination parameters for list endpoints.
#[derive(Debug, Clone, Default)]
pub struct PaginationParams {
    /// Number of results to return per page.
    /// Defaults to 20 if not specified. Maximum is 100.
    pub page_size: Option<i32>,
    /// Cursor for pagination.
    /// Use [`FIRST_PAGE_CURSOR`] for the first page of results.
    pub cursor: Option<String>,
    /// Sort specification as an ordered list of fields.
    pub sort: Vec<SortField>,
}

impl PaginationParams {
    /// Creates new pagination params with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the page size.
    pub fn with_page_size(mut self, page_size: i32) -> Self {
        self.page_size = Some(page_size);
        self
    }

    /// Sets the cursor.
    pub fn with_cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    /// Adds a sort field.
    pub fn with_sort(mut self, sort: SortField) -> Self {
        self.sort.push(sort);
        self
    }

    /// Sets the sort fields, replacing any existing ones.
    pub fn with_sort_fields(mut self, sort: Vec<SortField>) -> Self {
        self.sort = sort;
        self
    }
}

/// Pagination metadata for list responses.
#[derive(Debug, Clone, Default)]
pub struct PaginationMeta {
    /// Cursor for fetching the next page of results.
    /// When this equals the input cursor, there are no more results.
    pub next_cursor: Option<String>,
    /// Total number of items matching the request.
    /// May be omitted for performance reasons.
    pub total_count: Option<i64>,
}
