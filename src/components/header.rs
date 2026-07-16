use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Header() -> impl IntoView {
    view! {
        <header class="site-header">
            <A href="/" attr:class="brand-link">
                <img class="brand-logo" src="/images/logo.png" alt="MeetCal" />
                <span>"MeetCal"</span>
            </A>
            <nav class="site-nav" aria-label="Primary navigation">
                <A href="/privacy">"Privacy"</A>
                <A href="/terms">"Terms"</A>
                <A href="mailto:maddisen@meetcal.app">"Contact Us"</A>
            </nav>
            <A href="https://accounts.meetcal.app/sign-in" attr:class="profile-button">
                <span class="profile-icon" aria-hidden="true"></span>
                <span>"Profile"</span>
            </A>
        </header>
    }
}
