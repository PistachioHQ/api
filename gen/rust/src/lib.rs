/// Pistachio API proto definitions.
///
/// This crate provides the generated Rust types for the Pistachio API,
/// including both client and server implementations.
pub mod pistachio {
    /// Common types shared across APIs.
    pub mod types {
        pub mod v1 {
            tonic::include_proto!("pistachio.types.v1");
        }
    }

    /// Public API for client applications.
    pub mod v1 {
        tonic::include_proto!("pistachio.v1");
    }

    /// Admin API for service account operations.
    pub mod admin {
        pub mod v1 {
            tonic::include_proto!("pistachio.admin.v1");
        }
    }
}
