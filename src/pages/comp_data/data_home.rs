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
                "Browse meets, team performance, athlete history, qualification data, rankings, and records."
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
                <A href="/meet-center" attr:class="data-card">
                    <h2>"Meets"</h2>
                    <p>"Venue details, schedules, start lists, and complete meet results."</p>
                </A>
                <A href="/club-dashboard" attr:class="data-card">
                    <h2>"Club Dashboard"</h2>
                    <p>"Meet performance, medals, PRs, make rates, and athlete totals by club."</p>
                </A>
                <A href="/wso-dashboard" attr:class="data-card">
                    <h2>"WSO Dashboard"</h2>
                    <p>"Participation, make rates, lifted volume, and results for a selected WSO."</p>
                </A>
                <A href="/wrapped" attr:class="data-card">
                    <h2>"Athlete Wrapped"</h2>
                    <p>"Yearly athlete recaps with best lifts, make rate, and progress."</p>
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
