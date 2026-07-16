use leptos::prelude::*;
use serde::Deserialize;
use wasm_bindgen::{JsCast, JsValue, closure::Closure, prelude::wasm_bindgen};
use wasm_bindgen_futures::JsFuture;

pub const CLERK_PUBLISHABLE_KEY: Option<&str> = option_env!("CLERK_PUBLISHABLE_KEY");
pub const REVENUECAT_PUBLIC_API_KEY: Option<&str> = option_env!("REVENUECAT_PUBLIC_API_KEY");

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthState {
    Loading,
    SignedOut,
    SignedIn(String),
    ConfigurationError(String),
}

#[derive(Clone, Copy)]
pub struct AuthContext {
    pub state: ReadSignal<AuthState>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PurchasePackage {
    pub identifier: String,
    pub title: String,
    pub description: Option<String>,
    pub formatted_price: String,
    pub period_label: String,
}

#[wasm_bindgen(inline_js = r##"
const CLERK_UI_VERSION = "1";
const CLERK_JS_VERSION = "6";
const REVENUECAT_JS_VERSION = "1.47.3";

function loadScript(src, attributes = {}) {
  return new Promise((resolve, reject) => {
    const existing = document.querySelector(`script[src="${src}"]`);
    if (existing?.dataset.loaded === "true") {
      resolve();
      return;
    }

    const script = existing || document.createElement("script");
    Object.entries(attributes).forEach(([name, value]) => script.setAttribute(name, value));
    script.src = src;
    script.defer = true;
    script.crossOrigin = "anonymous";
    script.addEventListener("load", () => {
      script.dataset.loaded = "true";
      resolve();
    }, { once: true });
    script.addEventListener("error", () => reject(new Error(`Failed to load ${src}`)), { once: true });
    if (!existing) document.head.appendChild(script);
  });
}

function clerkUserId() {
  return window.Clerk?.isSignedIn && window.Clerk.user ? window.Clerk.user.id : null;
}

export async function initialize_clerk(publishableKey, onAuthChanged) {
  if (!publishableKey) throw new Error("CLERK_PUBLISHABLE_KEY is not configured");

  const encodedDomain = publishableKey.split("_")[2];
  if (!encodedDomain) throw new Error("CLERK_PUBLISHABLE_KEY is invalid");
  const clerkDomain = atob(encodedDomain).slice(0, -1);
  const uiUrl = `https://${clerkDomain}/npm/@clerk/ui@${CLERK_UI_VERSION}/dist/ui.browser.js`;
  const clerkUrl = `https://${clerkDomain}/npm/@clerk/clerk-js@${CLERK_JS_VERSION}/dist/clerk.browser.js`;

  await loadScript(uiUrl);
  await loadScript(clerkUrl, { "data-clerk-publishable-key": publishableKey });
  await window.Clerk.load({
    ui: { ClerkUI: window.__internal_ClerkUICtor },
    appearance: {
      variables: {
        colorPrimary: "#007aff",
        colorText: "#10223b",
        borderRadius: "0.85rem",
        fontFamily: "Inter, ui-sans-serif, system-ui, sans-serif",
      },
      options: {
        privacyPageUrl: "/privacy",
        termsPageUrl: "/terms",
      },
    },
  });

  window.Clerk.addListener(() => onAuthChanged(clerkUserId()));
  onAuthChanged(clerkUserId());
}

export function mount_clerk_user_button(element, revenueCatApiKey, appUserId) {
  if (!window.Clerk?.isSignedIn) return;
  element.replaceChildren();
  window.Clerk.mountUserButton(element, {
    showName: true,
    userProfileMode: "modal",
    appearance: {
      elements: {
        userButtonBox: { flexDirection: "row-reverse" },
        userButtonOuterIdentifier: { fontWeight: "700", color: "#10223b" },
      },
    },
    customMenuItems: [{
      label: "Manage subscription",
      onClick: async () => {
        try {
          const purchases = await revenueCatForUser(revenueCatApiKey, appUserId);
          const customerInfo = await purchases.getCustomerInfo();
          if (!customerInfo.managementURL) {
            throw new Error("No manageable subscription was found for this account.");
          }
          window.location.assign(customerInfo.managementURL);
        } catch (error) {
          window.alert(error?.message || "Subscription management is temporarily unavailable.");
        }
      },
      mountIcon: icon => {
        icon.innerHTML = `<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M4 7h16M7 11h2m3 0h2"/><rect x="3" y="5" width="18" height="14" rx="3"/></svg>`;
      },
      unmountIcon: icon => { if (icon) icon.replaceChildren(); },
    }],
  });
}

export function unmount_clerk_user_button(element) {
  if (window.Clerk) window.Clerk.unmountUserButton(element);
}

export function mount_clerk_sign_in(element) {
  element.replaceChildren();
  window.Clerk.mountSignIn(element, {
    appearance: { elements: { rootBox: { margin: "0 auto" } } },
  });
}

export function unmount_clerk_sign_in(element) {
  if (window.Clerk) window.Clerk.unmountSignIn(element);
}

export function open_clerk_sign_in() {
  if (window.Clerk) window.Clerk.openSignIn();
}

async function revenueCatForUser(apiKey, appUserId) {
  if (!apiKey) throw new Error("REVENUECAT_PUBLIC_API_KEY is not configured");
  await loadScript(`https://unpkg.com/@revenuecat/purchases-js@${REVENUECAT_JS_VERSION}/dist/Purchases.umd.js`);

  const Purchases = window.Purchases?.Purchases;
  if (!Purchases) throw new Error("RevenueCat Web SDK did not initialize");

  if (!Purchases.isConfigured()) {
    return Purchases.configure({ apiKey, appUserId });
  }

  const purchases = Purchases.getSharedInstance();
  if (purchases.getAppUserId() !== appUserId) await purchases.changeUser(appUserId);
  return purchases;
}

function hasActiveEntitlement(customerInfo) {
  return Object.keys(customerInfo.entitlements.active).length > 0;
}

function offeringWithWebPackages(offerings) {
  if (offerings.current?.availablePackages?.length) return offerings.current;
  return Object.values(offerings.all).find(offering => offering.availablePackages?.length) ?? null;
}

function periodLabel(duration, productType) {
  const labels = {
    P1W: "per week",
    P1M: "per month",
    P2M: "every 2 months",
    P3M: "every 3 months",
    P6M: "every 6 months",
    P1Y: "per year",
  };
  if (duration && labels[duration]) return labels[duration];
  return productType === "subscription" ? "subscription" : "one-time purchase";
}

export async function check_revenuecat_entitlement(apiKey, appUserId) {
  const purchases = await revenueCatForUser(apiKey, appUserId);
  const customerInfo = await purchases.getCustomerInfo();
  return hasActiveEntitlement(customerInfo);
}

export async function get_revenuecat_packages(apiKey, appUserId) {
  const purchases = await revenueCatForUser(apiKey, appUserId);
  const offering = offeringWithWebPackages(await purchases.getOfferings());
  if (!offering) return "[]";

  return JSON.stringify(offering.availablePackages.map(rcPackage => {
    const product = rcPackage.webBillingProduct;
    return {
      identifier: rcPackage.identifier,
      title: product.title || product.displayName || "MeetCal subscription",
      description: product.description || null,
      formattedPrice: product.currentPrice.formattedPrice,
      periodLabel: periodLabel(product.normalPeriodDuration, product.productType),
    };
  }));
}

export async function purchase_revenuecat_package(apiKey, appUserId, packageIdentifier) {
  const purchases = await revenueCatForUser(apiKey, appUserId);
  const offering = offeringWithWebPackages(await purchases.getOfferings());
  const rcPackage = offering?.availablePackages.find(pkg => pkg.identifier === packageIdentifier);
  if (!rcPackage) throw new Error("That subscription plan is no longer available. Refresh and try again.");

  const customerEmail = window.Clerk?.user?.primaryEmailAddress?.emailAddress;
  const result = await purchases.purchase({
    rcPackage,
    customerEmail,
    skipSuccessPage: true,
  });
  return hasActiveEntitlement(result.customerInfo);
}
"##)]
extern "C" {
    fn initialize_clerk(publishable_key: &str, callback: &js_sys::Function) -> js_sys::Promise;

    fn mount_clerk_user_button(
        element: &web_sys::HtmlElement,
        revenuecat_api_key: &str,
        app_user_id: &str,
    );
    fn unmount_clerk_user_button(element: &web_sys::HtmlElement);
    fn mount_clerk_sign_in(element: &web_sys::HtmlElement);
    fn unmount_clerk_sign_in(element: &web_sys::HtmlElement);
    pub fn open_clerk_sign_in();

    fn check_revenuecat_entitlement(api_key: &str, app_user_id: &str) -> js_sys::Promise;

    fn get_revenuecat_packages(api_key: &str, app_user_id: &str) -> js_sys::Promise;

    fn purchase_revenuecat_package(
        api_key: &str,
        app_user_id: &str,
        package_identifier: &str,
    ) -> js_sys::Promise;
}

pub fn provide_auth() {
    let (state, set_state) = signal(AuthState::Loading);
    provide_context(AuthContext { state });

    let Some(publishable_key) = CLERK_PUBLISHABLE_KEY else {
        set_state.set(AuthState::ConfigurationError(
            "Clerk is not configured for this deployment.".to_owned(),
        ));
        return;
    };

    let callback = Closure::<dyn Fn(JsValue)>::new(move |user_id: JsValue| {
        if let Some(user_id) = user_id.as_string() {
            set_state.set(AuthState::SignedIn(user_id));
        } else {
            set_state.set(AuthState::SignedOut);
        }
    });

    let initialization = initialize_clerk(publishable_key, callback.as_ref().unchecked_ref());
    callback.forget();

    leptos::task::spawn_local(async move {
        if let Err(error) = JsFuture::from(initialization).await {
            set_state.set(AuthState::ConfigurationError(js_error_message(error)));
        }
    });
}

pub fn mount_user_button(element: &web_sys::HtmlElement, app_user_id: &str) {
    mount_clerk_user_button(
        element,
        REVENUECAT_PUBLIC_API_KEY.unwrap_or_default(),
        app_user_id,
    );
}

pub fn unmount_user_button(element: &web_sys::HtmlElement) {
    unmount_clerk_user_button(element);
}

pub fn mount_sign_in(element: &web_sys::HtmlElement) {
    mount_clerk_sign_in(element);
}

pub fn unmount_sign_in(element: &web_sys::HtmlElement) {
    unmount_clerk_sign_in(element);
}

pub async fn has_comp_data_access(app_user_id: &str) -> Result<bool, String> {
    let api_key = REVENUECAT_PUBLIC_API_KEY
        .ok_or_else(|| "RevenueCat is not configured for this deployment.".to_owned())?;
    JsFuture::from(check_revenuecat_entitlement(api_key, app_user_id))
        .await
        .map(|value| value.as_bool().unwrap_or(false))
        .map_err(js_error_message)
}

pub async fn get_purchase_packages(app_user_id: &str) -> Result<Vec<PurchasePackage>, String> {
    let api_key = REVENUECAT_PUBLIC_API_KEY
        .ok_or_else(|| "RevenueCat is not configured for this deployment.".to_owned())?;
    let value = JsFuture::from(get_revenuecat_packages(api_key, app_user_id))
        .await
        .map_err(js_error_message)?;
    let json = value
        .as_string()
        .ok_or_else(|| "RevenueCat returned an invalid product catalog.".to_owned())?;
    serde_json::from_str(&json)
        .map_err(|_| "RevenueCat returned an invalid product catalog.".to_owned())
}

pub async fn purchase_package(app_user_id: &str, package_identifier: &str) -> Result<bool, String> {
    let api_key = REVENUECAT_PUBLIC_API_KEY
        .ok_or_else(|| "RevenueCat is not configured for this deployment.".to_owned())?;
    JsFuture::from(purchase_revenuecat_package(
        api_key,
        app_user_id,
        package_identifier,
    ))
    .await
    .map(|value| value.as_bool().unwrap_or(false))
    .map_err(js_error_message)
}

fn js_error_message(error: JsValue) -> String {
    js_sys::Reflect::get(&error, &JsValue::from_str("message"))
        .ok()
        .and_then(|message| message.as_string())
        .or_else(|| error.as_string())
        .unwrap_or_else(|| "The authentication service could not be reached.".to_owned())
}
