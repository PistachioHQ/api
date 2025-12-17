/// Trait for converting JSON response types to domain types.
pub(crate) trait FromJson<T>: Sized {
    type Error;
    fn from_json(json: T) -> Result<Self, Self::Error>;
}
