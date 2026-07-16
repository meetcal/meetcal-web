use crate::components::feature::FeatureSection;
use crate::components::{footer::Footer, header::Header};
use leptos::prelude::*;

#[component]
pub fn Home() -> impl IntoView {
    view! {
        <Header />
        <TitleSection />
        <BodySection />
        <HighlightSection />
        <Footer />
    }
}

#[component]
pub fn TitleSection() -> impl IntoView {
    view! {
        <h1>"Your Competition Schedule, Simplified"</h1>
        <h4>"MeetCal transforms complex Olympic weightlifting competition schedules into an intuitive, easy-to-use mobile experience."</h4>

        <a href="https://apps.apple.com/us/app/meetcal/id6741133286">
            <button>"Download for iOS"</button>
        </a>
        <a href="https://play.google.com/store/apps/details?id=com.memohnsen.meetcal">
            <button>"Download for Android"</button>
        </a>
    }
}

#[component]
pub fn BodySection() -> impl IntoView {
    view! {
        <h2>"Everything you need to stay on track"</h2>

        <FeatureSection title="Simple Schedule View" subtitle="Access your competition schedule instantly without opening a single PDF." />

        <FeatureSection title="Save to Calendar" subtitle="Save sessions within the app or to your calendar to always know where to be." />

        <FeatureSection title="Track Your Team" subtitle="Keep tabs on all your athletes and teams in one place." />

        <FeatureSection title="Smart Filtering" subtitle="Filter by weight class, platform, or club to find exactly what you need." />

        <FeatureSection title="Cross-Platform" subtitle="Available on both iOS and Android, supporting both mobile and tablets." />
    }
}

#[component]
pub fn HighlightSection() -> impl IntoView {
    let screenshots = ["01", "02", "03", "04", "05", "06", "07", "08", "09", "10"];

    view! {
        <section class="highlight-section">
            <div class="highlight-copy">
                <p class="highlight-eyebrow">"Built for meet day"</p>
                <h2>"Throw Out the PDFs"</h2>
                <p>"Experience MeetCal's intuitive interface designed for athletes, coaches, and spectators."</p>
            </div>
            <div class="screenshot-marquee" aria-label="A scrolling preview of MeetCal app screens">
                <div class="screenshot-track">
                    <div class="screenshot-set">
                        {screenshots.iter().map(|screenshot| view! {
                            <img src=format!("/images/{screenshot}.png") alt="" />
                        }).collect_view()}
                    </div>
                    <div class="screenshot-set" aria-hidden="true">
                        {screenshots.iter().map(|screenshot| view! {
                            <img src=format!("/images/{screenshot}.png") alt="" />
                        }).collect_view()}
                    </div>
                </div>
            </div>
        </section>
    }
}
