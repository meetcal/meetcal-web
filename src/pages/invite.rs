use crate::components::{footer::Footer, header::Header};
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

const APP_STORE_URL: &str = "https://apps.apple.com/us/app/meetcal/id6741133286";
const PLAY_STORE_URL: &str = "https://play.google.com/store/apps/details?id=com.memohnsen.meetcal";

/// Normalize the raw `:code` route param into a valid invite code, or `None`
/// when it is missing or malformed. We only accept short alphanumeric codes
/// (optionally with `-` or `_`) so a junk URL falls back to the plain
/// download CTA instead of rendering a broken code block.
fn sanitize_code(raw: &str) -> Option<String> {
    let trimmed = raw.trim();

    if trimmed.is_empty() || trimmed.len() > 32 {
        return None;
    }

    let is_valid = trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');

    if is_valid {
        Some(trimmed.to_ascii_uppercase())
    } else {
        None
    }
}

#[component]
pub fn InvitePage() -> impl IntoView {
    let params = use_params_map();
    let code = move || {
        params
            .read()
            .get("code")
            .and_then(|raw| sanitize_code(&raw))
    };

    view! {
        <Header />
        <main class="invite-page">
            <section class="invite-hero" aria-labelledby="invite-title">
                <p class="hero-eyebrow">"You're invited to MeetCal"</p>
                <h1 id="invite-title">"A friend invited you to MeetCal"</h1>
                <p class="invite-copy">
                    "MeetCal turns Olympic weightlifting competition schedules into a clear, "
                    "meet-day experience for athletes, coaches, and spectators. Download the app to get started."
                </p>

                {move || match code() {
                    Some(invite_code) => view! { <InviteCodeBlock code=invite_code /> }.into_any(),
                    None => ().into_any(),
                }}

                <div class="store-links invite-store-links" aria-label="Download the MeetCal mobile app">
                    <a class="store-link store-link-primary" href=APP_STORE_URL>
                        <span>"Download on the App Store"</span>
                    </a>
                    <a class="store-link store-link-secondary" href=PLAY_STORE_URL>
                        <span>"Get it on Google Play"</span>
                    </a>
                </div>

                <p class="invite-note">
                    "Your friend gets the app's normal offer when they download it. When friends you invite "
                    "subscribe and stay subscribed, you earn rewards. Referral rewards are available through "
                    "the mobile app only."
                </p>
            </section>
        </main>
        <Footer />
    }
}

#[component]
fn InviteCodeBlock(code: String) -> impl IntoView {
    let (copied, set_copied) = signal(false);

    let code_for_copy = code.clone();
    let deep_link = format!("meetcal://invite/{code}");

    view! {
        <div class="invite-code-card">
            <p class="invite-code-label">"Enter this code when you sign up in the app"</p>
            <div class="invite-code-box">
                <code class="invite-code-value">{code.clone()}</code>
                <button
                    class="copy-command-button invite-copy-button"
                    type="button"
                    aria-label=move || if copied.get() {
                        "Invite code copied".to_string()
                    } else {
                        "Copy invite code".to_string()
                    }
                    on:click=move |_| {
                        let _ = leptos::web_sys::window()
                            .expect("window should be available")
                            .navigator()
                            .clipboard()
                            .write_text(&code_for_copy);

                        set_copied.set(true);
                        leptos::leptos_dom::helpers::set_timeout(
                            move || set_copied.set(false),
                            std::time::Duration::from_millis(1800),
                        );
                    }
                >
                    <svg
                        class=move || if copied.get() { "copy-icon is-hidden" } else { "copy-icon" }
                        viewBox="0 0 24 24"
                        aria-hidden="true"
                    >
                        <rect x="9" y="9" width="11" height="11" rx="2"></rect>
                        <path d="M15 9V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v7a2 2 0 0 0 2 2h3"></path>
                    </svg>
                    <svg
                        class=move || if copied.get() { "check-icon" } else { "check-icon is-hidden" }
                        viewBox="0 0 24 24"
                        aria-hidden="true"
                    >
                        <path d="m5 12 4 4L19 6"></path>
                    </svg>
                </button>
            </div>
            <a class="invite-open-link" href=deep_link>
                "Already have the app? Open in app"
            </a>
        </div>
    }
}
