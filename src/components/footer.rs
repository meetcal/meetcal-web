use js_sys::Date;
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Footer() -> impl IntoView {
    let current_date = Date::new_0();

    let formatted_date = format!("{}", current_date.get_full_year(),);

    view! {
        <footer class="site-footer">
            <h2>"Ready to transform your competition experience?"</h2>
            <a class="footer-cta" href="https://apps.apple.com/us/app/meetcal/id6741133286">
                "Download Now"
            </a>

            <p>"© " {formatted_date} " MeetCal LLC. All rights reserved."</p>
            <nav class="footer-legal" aria-label="Legal">
                <A href="/privacy">"Privacy Policy"</A>
                <A href="/terms">"Terms of Use"</A>
            </nav>
        </footer>
    }
}
