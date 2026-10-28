use std::sync::Arc;

use crate::client::HttpClient;

pub mod analytics;
pub mod call_logs;
pub mod calls;
pub mod global_voice_settings;
pub mod number_routes;
pub mod programmable_voice_profiles;
pub mod sip_profiles;
pub mod sip_trunks;
pub mod tokens;

pub use analytics::{AnalyticsDomain, VoiceAnalyticsParams};
pub use call_logs::{CallLogsDomain, ListCallLogsParams};
pub use calls::{CallsDomain, ListCallsParams};
pub use global_voice_settings::GlobalVoiceSettingsDomain;
pub use number_routes::NumberRoutesDomain;
pub use programmable_voice_profiles::ProgrammableVoiceProfilesDomain;
pub use sip_profiles::SipProfilesDomain;
pub use sip_trunks::SipTrunksDomain;
pub use tokens::TokensDomain;

/// Voice domain — wraps the Voice service (mounted under `/voice`).
///
/// Sub-resources:
/// * `client.voice.sip_trunks`                 — SIP trunk (peer-to-peer connections to PBX/carrier).
/// * `client.voice.sip_profiles`               — SIP profile / endpoint (single registerable UA).
/// * `client.voice.programmable_voice_profiles` — Programmable Voice Profile (route a set of numbers through Signal House).
/// * `client.voice.number_routes`              — Per-number inbound route (overrides the profile and subgroup settings).
/// * `client.voice.calls`                      — Outbound call origination + call log queries.
/// * `client.voice.call_logs`                  — Account-scoped voice call history (cost, recording, attribution).
/// * `client.voice.analytics`                  — Aggregated call metrics (summary tiles + byDate/byNumber/byCarrier breakdowns).
/// * `client.voice.global_voice_settings`      — Account-wide voice defaults (accepted regions, max spend/min, E911).
/// * `client.voice.tokens`                     — Mint ephemeral SIP credentials for the browser voice SDK.
pub struct VoiceDomain {
    pub sip_trunks: SipTrunksDomain,
    pub sip_profiles: SipProfilesDomain,
    pub programmable_voice_profiles: ProgrammableVoiceProfilesDomain,
    pub number_routes: NumberRoutesDomain,
    pub calls: CallsDomain,
    pub call_logs: CallLogsDomain,
    pub analytics: AnalyticsDomain,
    pub global_voice_settings: GlobalVoiceSettingsDomain,
    pub tokens: TokensDomain,
}

impl VoiceDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self {
            sip_trunks: SipTrunksDomain::new(Arc::clone(&http)),
            sip_profiles: SipProfilesDomain::new(Arc::clone(&http)),
            programmable_voice_profiles: ProgrammableVoiceProfilesDomain::new(Arc::clone(&http)),
            number_routes: NumberRoutesDomain::new(Arc::clone(&http)),
            calls: CallsDomain::new(Arc::clone(&http)),
            call_logs: CallLogsDomain::new(Arc::clone(&http)),
            analytics: AnalyticsDomain::new(Arc::clone(&http)),
            global_voice_settings: GlobalVoiceSettingsDomain::new(Arc::clone(&http)),
            tokens: TokensDomain::new(http),
        }
    }
}
