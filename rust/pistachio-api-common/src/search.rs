//! Common search types.

use crate::pagination::{PaginationParams, SortField};

/// Common search parameters for search endpoints.
/// Extends PaginationParams with a query field.
#[derive(Debug, Clone, Default)]
pub struct SearchParams {
    /// Query string for searching.
    /// Empty string or "*" returns all documents.
    pub query: String,
    /// Pagination parameters.
    pub pagination: PaginationParams,
}

impl SearchParams {
    /// Creates new search params with an empty query.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the search query.
    pub fn with_query(mut self, query: impl Into<String>) -> Self {
        self.query = query.into();
        self
    }

    /// Sets the pagination parameters.
    pub fn with_pagination(mut self, pagination: PaginationParams) -> Self {
        self.pagination = pagination;
        self
    }

    /// Sets the page size.
    pub fn with_page_size(mut self, page_size: i32) -> Self {
        self.pagination.page_size = Some(page_size);
        self
    }

    /// Sets the cursor.
    pub fn with_cursor(mut self, cursor: impl Into<String>) -> Self {
        self.pagination.cursor = Some(cursor.into());
        self
    }

    /// Adds a sort field.
    pub fn with_sort(mut self, sort: SortField) -> Self {
        self.pagination.sort.push(sort);
        self
    }
}
