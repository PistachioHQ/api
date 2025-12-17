/// Pisachio API proto definitions.
///
/// This crate provides the generated Rust types for the pistachio.v1 API,
/// including both client and server implementations.
pub mod pistachio {
    pub mod v1 {
        tonic::include_proto!("pistachio.v1");
    }
}

pub use pistachio::v1::*;
