//! # SignalHouse Rust SDK
//!
//! Official Rust client library for the [SignalHouse](https://signalhouse.io)
//! SMS/MMS messaging platform.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use signalhouse_sms::{SignalHouseClient, SignalHouseConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), signalhouse_sms::SignalHouseError> {
//!     let client = SignalHouseClient::new(SignalHouseConfig {
//!         api_key: "your_api_key".into(),
//!         base_url: "https://v2.signalhouse.io".into(),
//!     })?;
//!
//!     // Login
//!     let resp = client.auth.login("user@example.com", "s3cret", None).await?;
//!     println!("{:?}", resp.data);
//!
//!     // Send an SMS
//!     let resp = client.messages.send_sms(
//!         "+15551234567",
//!         &serde_json::json!(["+15559876543"]),
//!         "Hello from Rust!",
//!         None,
//!         false,
//!         None,
//!     ).await?;
//!     println!("sent: {:?}", resp.success);
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod domains;
pub mod error;
pub mod request_options;
pub mod response;

// Re-export the most commonly used types at crate root.
pub use client::{SignalHouseClient, SignalHouseConfig};
pub use error::SignalHouseError;
pub use request_options::RequestOptions;
pub use response::SignalHouseResponse;
