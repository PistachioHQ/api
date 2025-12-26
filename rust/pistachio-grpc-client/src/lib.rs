mod types;

mod admin;
pub use admin::AdminClient;

/// Re-export tonic Channel for use with `AdminClient::from_channel`.
pub use tonic::transport::Channel;
