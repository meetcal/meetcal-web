use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;
use meetcal_web::pages::{
    comp_data::{
        data_home::CompData, qual_totals::QualifyingTotals, rankings::Rankings, records::Records,
        results::Results, standards::Standards,
    },
    features::FeaturesPage,
    home::Home,
    invite::InvitePage,
    not_found::NotFound,
    privacy::PrivacyPage,
    terms::TermsPage,
};
use meetcal_web::{auth::provide_auth, components::subscription_gate::SubscriptionGate};

fn main() {
    leptos::mount::mount_to_body(App)
}

#[component]
fn App() -> impl IntoView {
    provide_auth();

    view! {
        <Router>
            <main>
                <Routes fallback=|| view! { <NotFound /> }>
                    <Route path=path!("/") view=Home />
                    <Route path=path!("/comp-data") view=|| view! { <SubscriptionGate><CompData /></SubscriptionGate> } />
                    <Route path=path!("/qualifying-totals") view=|| view! { <SubscriptionGate><QualifyingTotals /></SubscriptionGate> } />
                    <Route path=path!("/standards") view=|| view! { <SubscriptionGate><Standards /></SubscriptionGate> } />
                    <Route path=path!("/records") view=|| view! { <SubscriptionGate><Records /></SubscriptionGate> } />
                    <Route path=path!("/results") view=|| view! { <SubscriptionGate><Results /></SubscriptionGate> } />
                    <Route path=path!("/rankings") view=|| view! { <SubscriptionGate><Rankings /></SubscriptionGate> } />
                    <Route path=path!("/features") view=FeaturesPage />
                    <Route path=path!("/invite") view=InvitePage />
                    <Route path=path!("/invite/:code") view=InvitePage />
                    <Route path=path!("/privacy") view=PrivacyPage />
                    <Route path=path!("/terms") view=TermsPage />
                </Routes>
            </main>
        </Router>
    }
}
