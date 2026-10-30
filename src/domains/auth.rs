use std::sync::Arc;

use reqwest::Method;
use serde_json::json;

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// Authentication domain -- login, password reset, and auth history.
pub struct AuthDomain {
    http: Arc<HttpClient>,
}

impl AuthDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// Login with email and password. Returns `{ token }`, or `{ mfaRequired: true, challengeId, ... }`
    /// when the account has 2FA on, or `{ token, twoFactorSetupRequired: true }` (a setup-only session)
    /// when the user's group requires 2FA and they haven't turned it on.
    ///
    /// **POST** `/auth`
    pub async fn login(
        &self,
        email: &str,
        password: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let body = json!({ "email": email, "password": password });
        self.http
            .request_json("/auth", Method::POST, Some(&body), options)
            .await
    }

    /// Reset a user's password.
    ///
    /// **PUT** `/auth/resetpassword/:userId`
    pub async fn reset_password(
        &self,
        user_id: &str,
        new_password: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("userId", Some(user_id)),
            ("newPassword", Some(new_password)),
        ])?;
        let safe = urlencoding::encode(user_id);
        let body = json!({ "newPassword": new_password });
        self.http
            .request_json(
                &format!("/auth/resetpassword/{}", safe),
                Method::PUT,
                Some(&body),
                options,
            )
            .await
    }

    /// Request a password-reset link be emailed to the given address (public).
    ///
    /// Always succeeds regardless of whether an account exists for the email, so it cannot
    /// be used to enumerate registered emails.
    ///
    /// **POST** `/auth/forgot-password`
    pub async fn forgot_password(
        &self,
        email: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("email", Some(email))])?;
        let body = json!({ "email": email });
        self.http
            .request_json("/auth/forgot-password", Method::POST, Some(&body), options)
            .await
    }

    /// Reset a password using the single-use token from a reset email (public).
    ///
    /// **POST** `/auth/reset-password`
    pub async fn reset_password_with_token(
        &self,
        token: &str,
        password: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("token", Some(token)), ("password", Some(password))])?;
        let body = json!({ "token": token, "password": password });
        self.http
            .request_json("/auth/reset-password", Method::POST, Some(&body), options)
            .await
    }

    /// Get token login history for a group or user.
    ///
    /// **GET** `/auth/history?...`
    pub async fn get_auth_history(
        &self,
        group_id: Option<&str>,
        user_id: Option<&str>,
        page: Option<u32>,
        limit: Option<u32>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let either = group_id.or(user_id);
        HttpClient::require(&[("groupId or userId", either)])?;

        let qs = HttpClient::get_query_string(&[
            ("groupId", group_id.map(String::from)),
            ("userId", user_id.map(String::from)),
            ("page", page.map(|p| p.to_string())),
            ("limit", limit.map(|l| l.to_string())),
        ]);
        self.http
            .request_json(&format!("/auth/history{}", qs), Method::GET, None, options)
            .await
    }

    /// Log out all other users in the caller's active group.
    ///
    /// **POST** `/auth/logout-all`
    pub async fn logout_all(
        &self,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/auth/logout-all", Method::POST, None, options)
            .await
    }

    /// Get the Group ID associated with the caller's JWT (their active group).
    ///
    /// **GET** `/auth/group-id`
    pub async fn get_group_id(
        &self,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/auth/group-id", Method::GET, None, options)
            .await
    }

    /// Mint a single-use, short-lived external-link token for the authenticated caller.
    ///
    /// The token is handed to the GHL/Shopify backend so it can link the caller's existing
    /// V2 group to its tenant.
    ///
    /// **POST** `/auth/external-link-token`
    pub async fn request_external_link_token(
        &self,
        product: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("product", Some(product))])?;
        let body = json!({ "product": product });
        self.http
            .request_json(
                "/auth/external-link-token",
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// List the OAuth sign-in providers enabled on this API (only providers with credentials
    /// configured).
    ///
    /// **GET** `/auth/oauth/providers`
    pub async fn get_oauth_providers(
        &self,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/auth/oauth/providers", Method::GET, None, options)
            .await
    }

    /// Build the URL that starts an OAuth sign-in for `provider` ("google", "microsoft" or
    /// "apple").
    ///
    /// Navigate a browser to it; it is a redirect, not an API call. The API returns the user to
    /// the portal's `/auth/callback` with a single-use ticket.
    ///
    /// `next` is optional: a same-site path (e.g. "/oauth/consent?request=abc") to return the
    /// user to after sign-in. The API hands it back on the `/auth/callback` redirect as `next`;
    /// anything that isn't a rooted path on the portal is dropped.
    ///
    /// Equivalent to `get_oauth_authorization_url_with(provider, next, None, None)`; use that
    /// function for Enterprise SSO (`email`) or account linking (`link_ticket`).
    pub fn get_oauth_authorization_url(
        &self,
        provider: &str,
        next: Option<&str>,
    ) -> Result<String, SignalHouseError> {
        self.get_oauth_authorization_url_with(provider, next, None, None)
    }

    /// Build the URL that starts an OAuth sign-in for `provider` ("google", "microsoft",
    /// "apple" or "sso" for Enterprise SSO), with optional `email` and `link_ticket`.
    ///
    /// Navigate a browser to it; it is a redirect, not an API call. The API returns the user to
    /// the portal's `/auth/callback` with a single-use ticket. With a `link_ticket` (from
    /// `create_oauth_link`) it links the provider to the signed-in account instead, and the API
    /// returns the user to `/Settings/Security` with `pendingLink` (finish with
    /// `confirm_oauth_link`) or `linkError`.
    ///
    /// - `next`: a same-site path to return the user to after sign-in (see
    ///   `get_oauth_authorization_url`).
    /// - `email`: required for "sso" -- the user's work email, whose domain picks the
    ///   organization's identity provider.
    /// - `link_ticket`: link the provider to the signed-in account that requested this ticket.
    ///
    /// Query keys are emitted in the order `email`, `linkTicket`, `next`, only when set.
    pub fn get_oauth_authorization_url_with(
        &self,
        provider: &str,
        next: Option<&str>,
        email: Option<&str>,
        link_ticket: Option<&str>,
    ) -> Result<String, SignalHouseError> {
        HttpClient::require(&[("provider", Some(provider))])?;
        let non_empty = |v: Option<&str>| v.filter(|s| !s.is_empty()).map(String::from);
        let qs = HttpClient::get_query_string(&[
            ("email", non_empty(email)),
            ("linkTicket", non_empty(link_ticket)),
            ("next", non_empty(next)),
        ]);
        Ok(format!(
            "{}/auth/oauth/{}{}",
            self.http.base_url(),
            urlencoding::encode(provider),
            qs
        ))
    }

    /// Build the URL that signs the caller in again with `provider` ("google", "microsoft",
    /// "apple" or "sso"), already linked to their account, to confirm it's them. `reauth_ticket`
    /// comes from `create_oauth_reauth`; "sso" needs no email here.
    ///
    /// Navigate a browser to it; it is a redirect, not an API call. The API returns the user to
    /// `/Settings/Security` with a single-use `pendingReauth` (the session that started it redeems it with confirm_oauth_reauth) or `reauthError`. Kept separate from
    /// `get_oauth_authorization_url_with` so that function's signature doesn't change.
    ///
    /// Emits the single query key `reauthTicket`.
    pub fn get_oauth_reauth_authorization_url(
        &self,
        provider: &str,
        reauth_ticket: &str,
    ) -> Result<String, SignalHouseError> {
        HttpClient::require(&[
            ("provider", Some(provider)),
            ("reauthTicket", Some(reauth_ticket)),
        ])?;
        let qs = HttpClient::get_query_string(&[("reauthTicket", Some(reauth_ticket.to_string()))]);
        Ok(format!(
            "{}/auth/oauth/{}{}",
            self.http.base_url(),
            urlencoding::encode(provider),
            qs
        ))
    }

    /// List the caller's sign-in methods: whether they chose their own password
    /// (`passwordKnown` is false for an account created through a provider or Enterprise SSO),
    /// and every provider linked to the account.
    ///
    /// Returns `{ passwordKnown, links: [{ provider, email, linkedAt, lastLoginAt }] }`.
    ///
    /// **GET** `/auth/oauth/links`
    pub async fn get_oauth_links(
        &self,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/auth/oauth/links", Method::GET, None, options)
            .await
    }

    /// Start linking `provider` ("google", "microsoft" or "apple") to the caller's account.
    ///
    /// Returns a single-use link ticket (valid 2 minutes) as `{ linkTicket, expiresInSeconds }`:
    /// navigate the browser to `get_oauth_authorization_url_with(provider, None, None,
    /// Some(link_ticket))`. The round trip returns the browser to `/Settings/Security` with a
    /// single-use `pendingLink`, which must be passed to `confirm_oauth_link` to finish linking.
    /// Requires a fresh `confirm_two_factor_identity` from the same session, which this spends.
    ///
    /// **POST** `/auth/oauth/:provider/link`
    pub async fn create_oauth_link(
        &self,
        provider: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("provider", Some(provider))])?;
        let path = format!("/auth/oauth/{}/link", urlencoding::encode(provider));
        self.http
            .request_json(&path, Method::POST, None, options)
            .await
    }

    /// Start signing in again with `provider` ("google", "microsoft", "apple" or "sso"), which
    /// must already be linked to the caller's account, to confirm it's them. Spends nothing.
    ///
    /// Returns a single-use ticket (valid 2 minutes) as `{ reauthTicket, reauthSecret, expiresInSeconds }`.
    /// Keep `reauthSecret` in the client that started the flow: it is returned only here, and
    /// `confirm_oauth_reauth` needs it. Navigate the browser to `get_oauth_reauth_authorization_url(provider, reauth_ticket)`.
    /// The browser comes back to `/Settings/Security` with a single-use `pendingReauth` (the session that started it redeems it with confirm_oauth_reauth) or `reauthError`
    /// (reauth_wrong_account, reauth_unsupported, reauth_stale, invalid_state,
    /// sso_not_configured, access_denied or provider_error).
    ///
    /// **POST** `/auth/oauth/:provider/reauth`
    pub async fn create_oauth_reauth(
        &self,
        provider: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("provider", Some(provider))])?;
        let path = format!("/auth/oauth/{}/reauth", urlencoding::encode(provider));
        self.http
            .request_json(&path, Method::POST, None, options)
            .await
    }

    /// Finish signing in again with a linked provider. The round trip started with
    /// `create_oauth_reauth` returns the browser to `/Settings/Security?pendingReauth=...`; this
    /// redeems that single-use value (valid 5 minutes), but only for the same account and the same
    /// session that called `create_oauth_reauth`, so a re-sign-in completed in someone else's
    /// browser confirms nothing. Fails with 400 when it expired or was used, and 403 for another
    /// account or session.
    ///
    /// - `pending_reauth`: the `pendingReauth` query parameter from the redirect.
    /// - `reauth_secret`: the `reauthSecret` `create_oauth_reauth` returned; a wrong or missing
    ///   secret is refused (403) and uses up the pending re-sign-in.
    ///
    /// Returns `{ identityConfirmed }`: true when 2FA is off, so it counts as a confirmed identity.
    ///
    /// **POST** `/auth/oauth/reauth/confirm`
    pub async fn confirm_oauth_reauth(
        &self,
        pending_reauth: &str,
        reauth_secret: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("pendingReauth", Some(pending_reauth)),
            ("reauthSecret", Some(reauth_secret)),
        ])?;
        let body = json!({ "pendingReauth": pending_reauth, "reauthSecret": reauth_secret });
        self.http
            .request_json(
                "/auth/oauth/reauth/confirm",
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// Finish linking a provider. The provider round trip started with `create_oauth_link`
    /// returns the browser to `/Settings/Security?pendingLink=...`; this links it, but only for
    /// the account the link was started for, so a link URL opened in someone else's browser
    /// can't attach their provider to your account.
    ///
    /// - `pending_link`: the `pendingLink` query parameter from the redirect.
    ///
    /// Returns the caller's `{ passwordKnown, links: [{ provider, email, linkedAt, lastLoginAt }] }`.
    ///
    /// **POST** `/auth/oauth/links/confirm`
    pub async fn confirm_oauth_link(
        &self,
        pending_link: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("pendingLink", Some(pending_link))])?;
        let body = json!({ "pendingLink": pending_link });
        self.http
            .request_json(
                "/auth/oauth/links/confirm",
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// Unlink `provider` ("google", "microsoft", "apple" or "sso") from the caller's account.
    ///
    /// Refused (400) when it is the account's only way to sign in. Requires a fresh
    /// `confirm_two_factor_identity` from the same session. Returns the remaining
    /// `{ passwordKnown, links }`.
    ///
    /// **DELETE** `/auth/oauth/:provider/link`
    pub async fn delete_oauth_link(
        &self,
        provider: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("provider", Some(provider))])?;
        let path = format!("/auth/oauth/{}/link", urlencoding::encode(provider));
        self.http
            .request_json(&path, Method::DELETE, None, options)
            .await
    }

    /// Read what a single-use OAuth ticket resolves to, without consuming it: a "session"
    /// (ready to exchange), a "select" (pick one of several accounts) or a "signup" (finish
    /// registration).
    ///
    /// **POST** `/auth/oauth/ticket`
    pub async fn read_oauth_ticket(
        &self,
        ticket: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("ticket", Some(ticket))])?;
        let body = json!({ "ticket": ticket });
        self.http
            .request_json("/auth/oauth/ticket", Method::POST, Some(&body), options)
            .await
    }

    /// Exchange a single-use "session" or "select" OAuth ticket for a JWT. Consumes the ticket.
    ///
    /// `user_id` is required for a "select" ticket: the chosen account from "accounts".
    ///
    /// **POST** `/auth/oauth/exchange`
    pub async fn exchange_oauth_ticket(
        &self,
        ticket: &str,
        user_id: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("ticket", Some(ticket))])?;
        let mut body = json!({ "ticket": ticket });
        if let Some(id) = user_id {
            body["userId"] = json!(id);
        }
        self.http
            .request_json("/auth/oauth/exchange", Method::POST, Some(&body), options)
            .await
    }

    /// Login with email and password, passing a trusted-device token from an earlier
    /// `verify_two_factor_challenge` so 2FA is skipped while it's valid.
    ///
    /// The response holds a `token`, or `mfaRequired` + `challengeId` when the account has 2FA on.
    ///
    /// **POST** `/auth`
    pub async fn login_with_device_token(
        &self,
        email: &str,
        password: &str,
        device_token: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let mut body = json!({ "email": email, "password": password });
        if let Some(t) = device_token {
            body["deviceToken"] = json!(t);
        }
        self.http
            .request_json("/auth", Method::POST, Some(&body), options)
            .await
    }

    /// Exchange an OAuth ticket like `exchange_oauth_ticket`, passing a trusted-device token so 2FA
    /// is skipped while it's valid. May return `mfaRequired` + `challengeId` instead of a token.
    ///
    /// **POST** `/auth/oauth/exchange`
    pub async fn exchange_oauth_ticket_with_device_token(
        &self,
        ticket: &str,
        user_id: Option<&str>,
        device_token: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("ticket", Some(ticket))])?;
        let mut body = json!({ "ticket": ticket });
        if let Some(id) = user_id {
            body["userId"] = json!(id);
        }
        if let Some(t) = device_token {
            body["deviceToken"] = json!(t);
        }
        self.http
            .request_json("/auth/oauth/exchange", Method::POST, Some(&body), options)
            .await
    }

    /// The caller's two-factor settings: methods (targets masked), default method, backup-code
    /// counts and trusted devices. Never returns secrets or codes.
    ///
    /// **GET** `/auth/2fa`
    pub async fn get_two_factor_settings(
        &self,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/auth/2fa", Method::GET, None, options)
            .await
    }

    /// Confirm it's really the account owner before a sensitive change:
    /// `start_two_factor_enrollment`, `remove_two_factor_method`, `regenerate_backup_codes`,
    /// `disable_two_factor`, `create_oauth_link`, `delete_oauth_link`, or
    /// `groups.save_sso_connection` / `groups.delete_sso_connection`. With 2FA on, pass a current
    /// `code` (or a backup code); with 2FA off, pass the account `password`. An account without
    /// its own password (`get_oauth_links` returns `passwordKnown: false`) instead passes the
    /// `code` from `send_two_factor_identity_code("email")` with `method` "email". A confirmation
    /// lasts 5 minutes, belongs to the session that made it, and unlocks exactly one change.
    ///
    /// **POST** `/auth/2fa/confirm-identity`
    pub async fn confirm_two_factor_identity(
        &self,
        password: Option<&str>,
        code: Option<&str>,
        method: Option<&str>,
        using_backup_code: Option<bool>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let mut body = json!({});
        if let Some(v) = password {
            body["password"] = json!(v);
        }
        if let Some(v) = code {
            body["code"] = json!(v);
        }
        if let Some(v) = method {
            body["method"] = json!(v);
        }
        if let Some(v) = using_backup_code {
            body["usingBackupCode"] = json!(v);
        }
        self.http
            .request_json(
                "/auth/2fa/confirm-identity",
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// Send a code by an enrolled `"sms"` or `"email"` method to use with
    /// `confirm_two_factor_identity` (not needed for "app"). With 2FA off, an account without a
    /// password of its own (`get_oauth_links` returns `passwordKnown: false`) passes `"email"` and
    /// confirms with the emailed code and `method` "email".
    ///
    /// **POST** `/auth/2fa/confirm-identity/send`
    pub async fn send_two_factor_identity_code(
        &self,
        method: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("method", Some(method))])?;
        let body = json!({ "method": method });
        self.http
            .request_json(
                "/auth/2fa/confirm-identity/send",
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// Start enrolling a two-factor method (`"app"`, `"sms"` or `"email"`). For "app" the response
    /// carries the setup secret and otpauth URI; for "sms" and "email" a code is sent.
    /// Requires a fresh `confirm_two_factor_identity` from the same session (403 otherwise); each
    /// confirmation unlocks one change.
    ///
    /// **POST** `/auth/2fa/enroll`
    pub async fn start_two_factor_enrollment(
        &self,
        method: &str,
        phone: Option<&str>,
        address: Option<&str>,
        device_name: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("method", Some(method))])?;
        let mut body = json!({ "method": method });
        if let Some(v) = phone {
            body["phone"] = json!(v);
        }
        if let Some(v) = address {
            body["address"] = json!(v);
        }
        if let Some(v) = device_name {
            body["deviceName"] = json!(v);
        }
        self.http
            .request_json("/auth/2fa/enroll", Method::POST, Some(&body), options)
            .await
    }

    /// Confirm a pending enrollment with its code. The first method turns 2FA on and returns
    /// backup codes, shown only this once.
    ///
    /// **POST** `/auth/2fa/enroll/:enrollmentId/verify`
    pub async fn confirm_two_factor_enrollment(
        &self,
        enrollment_id: &str,
        code: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("enrollmentId", Some(enrollment_id)), ("code", Some(code))])?;
        let body = json!({ "code": code });
        let path = format!(
            "/auth/2fa/enroll/{}/verify",
            urlencoding::encode(enrollment_id)
        );
        self.http
            .request_json(&path, Method::POST, Some(&body), options)
            .await
    }

    /// Choose which enrolled method is offered first at sign-in.
    ///
    /// **PUT** `/auth/2fa/primary`
    pub async fn set_two_factor_primary_method(
        &self,
        method: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("method", Some(method))])?;
        let body = json!({ "method": method });
        self.http
            .request_json("/auth/2fa/primary", Method::PUT, Some(&body), options)
            .await
    }

    /// Remove an enrolled two-factor method. Removing the last one turns 2FA off.
    /// Requires a fresh `confirm_two_factor_identity` from the same session (403 otherwise); each
    /// confirmation unlocks one change.
    ///
    /// **DELETE** `/auth/2fa/methods/:method`
    pub async fn remove_two_factor_method(
        &self,
        method: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("method", Some(method))])?;
        let path = format!("/auth/2fa/methods/{}", urlencoding::encode(method));
        self.http
            .request_json(&path, Method::DELETE, None, options)
            .await
    }

    /// Replace all backup codes with a new set, shown only once. The previous codes stop working.
    /// Requires a fresh `confirm_two_factor_identity` from the same session (403 otherwise); each
    /// confirmation unlocks one change.
    ///
    /// **POST** `/auth/2fa/backup-codes`
    pub async fn regenerate_backup_codes(
        &self,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/auth/2fa/backup-codes", Method::POST, None, options)
            .await
    }

    /// Stop trusting a device, so its next sign-in asks for the second factor again.
    ///
    /// **DELETE** `/auth/2fa/trusted-devices/:deviceId`
    pub async fn revoke_trusted_device(
        &self,
        device_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("deviceId", Some(device_id))])?;
        let path = format!(
            "/auth/2fa/trusted-devices/{}",
            urlencoding::encode(device_id)
        );
        self.http
            .request_json(&path, Method::DELETE, None, options)
            .await
    }

    /// Turn two-factor authentication off: removes every method, backup code and trusted device.
    /// Requires a fresh `confirm_two_factor_identity` from the same session (403 otherwise); each
    /// confirmation unlocks one change.
    ///
    /// **DELETE** `/auth/2fa`
    pub async fn disable_two_factor(
        &self,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/auth/2fa", Method::DELETE, None, options)
            .await
    }

    /// Whether the "secure your account" enrollment prompt is due for the caller.
    ///
    /// **GET** `/auth/2fa/prompt`
    pub async fn get_two_factor_prompt(
        &self,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/auth/2fa/prompt", Method::GET, None, options)
            .await
    }

    /// Dismiss the enrollment prompt until the next sign-in. It stops for good after three dismissals.
    ///
    /// **POST** `/auth/2fa/prompt/dismiss`
    pub async fn dismiss_two_factor_prompt(
        &self,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/auth/2fa/prompt/dismiss", Method::POST, None, options)
            .await
    }

    /// Send a sign-in code by `"sms"` or `"email"` for a pending two-factor challenge (not needed
    /// for "app").
    ///
    /// **POST** `/auth/2fa/challenge/:challengeId/send`
    pub async fn send_two_factor_challenge_code(
        &self,
        challenge_id: &str,
        method: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("challengeId", Some(challenge_id)),
            ("method", Some(method)),
        ])?;
        let body = json!({ "method": method });
        let path = format!(
            "/auth/2fa/challenge/{}/send",
            urlencoding::encode(challenge_id)
        );
        self.http
            .request_json(&path, Method::POST, Some(&body), options)
            .await
    }

    /// Verify a sign-in code (or backup code) for a pending challenge and receive the session
    /// JWT. With `remember_device` the response also carries a `deviceToken`.
    ///
    /// **POST** `/auth/2fa/challenge/:challengeId/verify`
    #[allow(clippy::too_many_arguments)]
    pub async fn verify_two_factor_challenge(
        &self,
        challenge_id: &str,
        code: &str,
        method: Option<&str>,
        using_backup_code: Option<bool>,
        remember_device: Option<bool>,
        device_name: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("challengeId", Some(challenge_id)), ("code", Some(code))])?;
        let mut body = json!({ "code": code });
        if let Some(v) = method {
            body["method"] = json!(v);
        }
        if let Some(v) = using_backup_code {
            body["usingBackupCode"] = json!(v);
        }
        if let Some(v) = remember_device {
            body["rememberDevice"] = json!(v);
        }
        if let Some(v) = device_name {
            body["deviceName"] = json!(v);
        }
        let path = format!(
            "/auth/2fa/challenge/{}/verify",
            urlencoding::encode(challenge_id)
        );
        self.http
            .request_json(&path, Method::POST, Some(&body), options)
            .await
    }

    /// Cancel a pending two-factor challenge.
    ///
    /// **DELETE** `/auth/2fa/challenge/:challengeId`
    pub async fn cancel_two_factor_challenge(
        &self,
        challenge_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("challengeId", Some(challenge_id))])?;
        let path = format!("/auth/2fa/challenge/{}", urlencoding::encode(challenge_id));
        self.http
            .request_json(&path, Method::DELETE, None, options)
            .await
    }

    // -- Admin methods (require "admin" feature) ----------------------------

    /// Turn off two-factor authentication for a user who has lost every factor (lockout
    /// recovery). Clears their methods, backup codes and trusted devices. Audited.
    ///
    /// **DELETE** `/auth/2fa/users/:userId`
    ///
    /// Requires the `admin` crate feature.
    #[cfg(feature = "admin")]
    pub async fn reset_user_two_factor_admin(
        &self,
        user_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("userId", Some(user_id))])?;
        let path = format!("/auth/2fa/users/{}", urlencoding::encode(user_id));
        self.http
            .request_json(&path, Method::DELETE, None, options)
            .await
    }
}
