# SignalHouse Rust SDK

Official Rust client library for the [SignalHouse](https://signalhouse.io) SMS/MMS messaging platform.

Full API reference and guides: https://app2.signalhouse.io/docs

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
signalhouse-sms = "1.5.1"
tokio = { version = "1", features = ["full"] }
serde_json = "1"
```

To enable admin-only endpoints (template management, campaign approval, etc.):

```toml
[dependencies]
signalhouse-sms = { version = "1.5.1", features = ["admin"] }
```

## Quick Start

```rust
use signalhouse_sms::{SignalHouseClient, SignalHouseConfig};

#[tokio::main]
async fn main() -> Result<(), signalhouse_sms::SignalHouseError> {
    let client = SignalHouseClient::new(SignalHouseConfig {
        api_key: "your_api_key".into(),
        base_url: "https://v2.signalhouse.io".into(),
    })?;

    // Login
    let resp = client.auth.login("user@example.com", "s3cret", None).await?;
    println!("Login success: {}", resp.success);

    // Send an SMS
    let resp = client.messages.send_sms(
        "+15551234567",                              // sender
        &serde_json::json!(["+15559876543"]),         // recipients
        "Hello from the Rust SDK!",                   // body
        None,                                         // status callback URL
        false,                                        // enable shortlink
        None,                                         // request options
    ).await?;
    println!("SMS sent: {:?}", resp.data);

    Ok(())
}
```

## Per-Request Authentication

Every method accepts an optional `RequestOptions` to override the bearer token
for a single call. This is useful when acting on behalf of a specific user:

```rust
use signalhouse_sms::RequestOptions;

let opts = RequestOptions {
    token: Some("user_specific_jwt".into()),
    ..Default::default()
};

let resp = client.users.get_users(None, None, None, Some(&opts)).await?;
```

Set `idempotency_key` on `RequestOptions` to make a create, update or delete safe to retry: the SDK sends it as the
`Idempotency-Key` header, and a retry with the same key and request replays the first response instead of running it
again. Keys are scoped to the authenticated user and kept for 24 hours.

## Domains

| Domain | Description |
|---|---|
| `auth` | Login, password reset, auth history |
| `billing` | Wallets, payment methods, funds, invoices, fees |
| `brands` | 10DLC brand registration and vetting, toll-free business verification |
| `campaigns` | 10DLC campaign registration |
| `groups` | Group management |
| `landings` | Brand/campaign landing pages |
| `messages` | Send SMS/MMS, message logs, analytics |
| `notifications` | In-app notifications |
| `numbers` | Phone number purchase, assignment, porting |
| `shortlinks` | URL shortener |
| `subgroups` | Subgroup management |
| `subscriptions` | Subscription plans and billing |
| `templates` | Versioned message templates (`/template`): drafts, publish, validate, media |
| `users` | User CRUD and notification preferences |
| `webhooks` | Webhook CRUD |

## Admin Feature

Methods that require SignalHouse staff roles are gated behind the `admin`
Cargo feature. These include:

- `brands.get_toll_free_isv_account` / `brands.set_toll_free_isv_account`
- `campaigns.approve_campaign` / `campaigns.reject_campaign`
- `groups.get_groups` / `groups.create_group` / `groups.delete_group`
- `subscriptions.get_templates` / `subscriptions.create_template` / `subscriptions.update_template` / `subscriptions.delete_template` / `subscriptions.create_custom_subscription`
- `users.get_users_admin` / `users.create_internal_user`

## License

ISC
