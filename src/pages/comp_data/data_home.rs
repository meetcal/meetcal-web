use crate::components::{footer::Footer, header::Header};
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn CompData() -> impl IntoView {
    view! {
        <Header />
        <section class="data-page data-home">
            <p class="data-eyebrow">"Competition data"</p>
            <h1>"Find the numbers you need"</h1>
            <p class="data-intro">
                "Browse official qualification information, standards, results, rankings, and records."
            </p>
            <div class="data-card-grid">
                <A href="/qualifying-totals" attr:class="data-card">
                    <h2>"Qualifying Totals"</h2>
                    <p>"Qualification totals by event, division, and weight class."</p>
                </A>
                <A href="/standards" attr:class="data-card">
                    <h2>"Standards"</h2>
                    <p>"A and B standards by division and weight class."</p>
                </A>
                <A href="/results" attr:class="data-card">
                    <h2>"Results"</h2>
                    <p>"Search an athlete’s competition results over a date range."</p>
                </A>
                <A href="/rankings" attr:class="data-card">
                    <h2>"Rankings"</h2>
                    <p>"International rankings, totals, and percentage scores."</p>
                </A>
                <A href="/national-rankings" attr:class="data-card">
                    <h2>"National Rankings"</h2>
                    <p>"USAW and USAMW rankings by division, with optional year filtering."</p>
                </A>
                <A href="/records" attr:class="data-card">
                    <h2>"Records"</h2>
                    <p>"Snatch, clean and jerk, and total records."</p>
                </A>
                <A href="/wso-records" attr:class="data-card">
                    <h2>"WSO Records"</h2>
                    <p>"State-organization records by division and weight class."</p>
                </A>
                <A href="/adaptive-records" attr:class="data-card">
                    <h2>"Adaptive Records"</h2>
                    <p>"Top adaptive performances by gender and weight class."</p>
                </A>
            </div>
        </section>
        <Footer />
    }
}
