use crate::{
    auth::{
        AuthContext, AuthState, PurchasePackage, get_purchase_packages, has_comp_data_access,
        mount_sign_in, purchase_package, unmount_sign_in,
    },
    components::{footer::Footer, header::Header},
};
use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
enum AccessState {
    Checking,
    SignedOut,
    Paid,
    Unpaid(String),
    Error(String),
}

#[component]
pub fn SubscriptionGate(children: ChildrenFn) -> impl IntoView {
    let auth = expect_context::<AuthContext>();
    let (access, set_access) = signal(AccessState::Checking);

    Effect::new(move |_| match auth.state.get() {
        AuthState::Loading => set_access.set(AccessState::Checking),
        AuthState::SignedOut => set_access.set(AccessState::SignedOut),
        AuthState::ConfigurationError(message) => set_access.set(AccessState::Error(message)),
        AuthState::SignedIn(user_id) => {
            set_access.set(AccessState::Checking);
            leptos::task::spawn_local(async move {
                let result = has_comp_data_access(&user_id).await;
                if auth.state.get_untracked() != AuthState::SignedIn(user_id.clone()) {
                    return;
                }

                match result {
                    Ok(true) => set_access.set(AccessState::Paid),
                    Ok(false) => set_access.set(AccessState::Unpaid(user_id)),
                    Err(message) => set_access.set(AccessState::Error(message)),
                }
            });
        }
    });

    view! {
        {move || match access.get() {
            AccessState::Checking => view! {
                <AccessShell>
                    <div class="access-status" role="status" aria-live="polite">
                        <span class="access-spinner" aria-hidden="true"></span>
                        <h1>"Checking your access"</h1>
                        <p>"Confirming your account and subscription…"</p>
                    </div>
                </AccessShell>
            }.into_any(),
            AccessState::SignedOut => view! {
                <AccessShell>
                    <div class="access-heading">
                        <p class="data-eyebrow">"Competition data"</p>
                        <h1>"Sign in to continue"</h1>
                        <p>"Use your MeetCal account before accessing subscription-only competition data."</p>
                    </div>
                    <ClerkSignIn />
                </AccessShell>
            }.into_any(),
            AccessState::Paid => children().into_any(),
            AccessState::Unpaid(user_id) => view! {
                <PurchasePage user_id />
            }.into_any(),
            AccessState::Error(message) => view! {
                <AccessShell>
                    <div class="access-status access-error" role="alert">
                        <h1>"We couldn’t verify your access"</h1>
                        <p>{message}</p>
                        <button class="access-button" type="button" on:click=move |_| {
                            if let Some(window) = web_sys::window() {
                                let _ = window.location().reload();
                            }
                        }>
                            "Try again"
                        </button>
                    </div>
                </AccessShell>
            }.into_any(),
        }}
    }
}

#[component]
fn AccessShell(children: Children) -> impl IntoView {
    view! {
        <Header />
        <div class="access-page">
            <section class="access-card">{children()}</section>
        </div>
        <Footer />
    }
}

#[component]
fn ClerkSignIn() -> impl IntoView {
    let container = NodeRef::<leptos::html::Div>::new();

    Effect::new(move |_| {
        if let Some(element) = container.get() {
            mount_sign_in(&element);
        }
    });
    on_cleanup(move || {
        if let Some(element) = container.get_untracked() {
            unmount_sign_in(&element);
        }
    });

    view! { <div class="clerk-sign-in" node_ref=container></div> }
}

#[component]
fn PurchasePage(user_id: String) -> impl IntoView {
    let (packages, set_packages) = signal(None::<Result<Vec<PurchasePackage>, String>>);
    let (purchasing, set_purchasing) = signal(None::<String>);
    let (error, set_error) = signal(None::<String>);

    let user_id_for_load = user_id.clone();
    Effect::new(move |_| {
        let user_id = user_id_for_load.clone();
        leptos::task::spawn_local(async move {
            set_packages.set(Some(get_purchase_packages(&user_id).await));
        });
    });

    view! {
        <AccessShell>
            <div class="access-heading">
                <p class="data-eyebrow">"MeetCal subscription"</p>
                <h1>"Unlock competition data"</h1>
                <p>"Choose a plan for complete access to MeetCal’s competition data tools."</p>
            </div>
            <div class="purchase-benefits" aria-label="Subscription benefits">
                <span>"Qualifying totals"</span>
                <span>"Standards"</span>
                <span>"Results and rankings"</span>
                <span>"Competition records"</span>
            </div>
            {move || match packages.get() {
                None => view! {
                    <div class="purchase-loading" role="status">
                        <span class="access-spinner" aria-hidden="true"></span>
                        <span>"Loading available plans…"</span>
                    </div>
                }.into_any(),
                Some(Err(message)) => view! {
                    <p class="access-inline-error" role="alert">{message}</p>
                }.into_any(),
                Some(Ok(packages)) if packages.is_empty() => view! {
                    <div class="purchase-empty" role="alert">
                        <h2>"Plans are not available yet"</h2>
                        <p>"No web products were found in your RevenueCat Offerings. Please check back shortly."</p>
                    </div>
                }.into_any(),
                Some(Ok(packages)) => view! {
                    <div class="purchase-plans">
                        {packages.into_iter().map(|package| {
                            let package_id = package.identifier.clone();
                            let package_id_for_state = package.identifier.clone();
                            let user_id = user_id.clone();
                            view! {
                                <article class="purchase-plan">
                                    <div class="purchase-plan-copy">
                                        <h2>{package.title}</h2>
                                        {package.description.map(|description| view! { <p>{description}</p> })}
                                    </div>
                                    <div class="purchase-plan-price">
                                        <strong>{package.formatted_price}</strong>
                                        <span>{package.period_label}</span>
                                    </div>
                                    <button
                                        class="access-button purchase-button"
                                        type="button"
                                        disabled=move || purchasing.get().is_some()
                                        on:click=move |_| {
                                            let user_id = user_id.clone();
                                            let package_id = package_id.clone();
                                            set_purchasing.set(Some(package_id.clone()));
                                            set_error.set(None);
                                            leptos::task::spawn_local(async move {
                                                match purchase_package(&user_id, &package_id).await {
                                                    Ok(true) => {
                                                        if let Some(window) = web_sys::window() {
                                                            let _ = window.location().reload();
                                                        }
                                                    }
                                                    Ok(false) => set_error.set(Some(
                                                        "The purchase was not completed. Your account has not been charged.".to_owned(),
                                                    )),
                                                    Err(message) => set_error.set(Some(message)),
                                                }
                                                set_purchasing.set(None);
                                            });
                                        }
                                    >
                                        {move || if purchasing.get().as_deref() == Some(package_id_for_state.as_str()) {
                                            "Opening checkout…"
                                        } else {
                                            "Choose plan"
                                        }}
                                    </button>
                                </article>
                            }
                        }).collect_view()}
                    </div>
                }.into_any(),
            }}
            {move || error.get().map(|message| view! {
                <p class="access-inline-error" role="alert">{message}</p>
            })}
            <p class="purchase-disclaimer">"Secure checkout is handled by RevenueCat. Subscription terms and renewal details are shown before purchase."</p>
        </AccessShell>
    }
}
