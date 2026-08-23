import AxeBuilder from "@axe-core/playwright";
import { expect, test, type Page } from "@playwright/test";

async function mockSubscribedUser(page: Page) {
  await page.route(/cdn\.jsdelivr\.net\/npm\/@clerk\/ui@/, async (route) => {
    await route.fulfill({
      contentType: "application/javascript",
      body: "window.__internal_ClerkUICtor = function ClerkUI() {};",
    });
  });
  await page.route(/cdn\.jsdelivr\.net\/npm\/@clerk\/clerk-js@/, async (route) => {
    await route.fulfill({
      contentType: "application/javascript",
      body: `window.Clerk = {
        isSignedIn: true,
        user: { id: "user_ci" },
        load: async () => {},
        addListener: () => {},
        mountUserButton: (element) => { element.textContent = "Test User"; },
        unmountUserButton: () => {},
        mountSignIn: () => {},
        unmountSignIn: () => {},
        openSignIn: () => {}
      };`,
    });
  });
  await page.route(/unpkg\.com\/@revenuecat\/purchases-js@/, async (route) => {
    await route.fulfill({
      contentType: "application/javascript",
      body: `(() => {
        const purchases = {
          getAppUserId: () => "user_ci",
          changeUser: async () => {},
          getCustomerInfo: async () => ({ entitlements: { active: { pro: {} } } })
        };
        window.Purchases = { Purchases: {
          isConfigured: () => false,
          configure: () => purchases,
          getSharedInstance: () => purchases
        }};
      })();`,
    });
  });
}

function jsonResponse(body: unknown) {
  return {
    contentType: "application/json",
    headers: { "access-control-allow-origin": "*" },
    body: JSON.stringify(body),
  };
}

const publicRoutes = [
  ["/", "Your Competition Schedule, Simplified"],
  ["/features", "Everything you need before the bar is loaded"],
  ["/privacy", "Privacy Policy"],
  ["/terms", "Terms of Use"],
  ["/subscription", "Continue in the MeetCal app"],
] as const;

const protectedDataRoutes = [
  "/comp-data",
  "/qualifying-totals",
  "/standards",
  "/results",
  "/rankings",
  "/national-rankings",
  "/records",
  "/wso-records",
  "/adaptive-records",
  "/meet-center",
  "/club-dashboard",
  "/wso-dashboard",
  "/wrapped",
] as const;

for (const [path, heading] of publicRoutes) {
  test(`${path} renders its primary content`, async ({ page }) => {
    await page.goto(path);

    await expect(page.getByRole("heading", { level: 1, name: heading })).toBeVisible();
    await expect(page.locator("header.site-header")).toBeVisible();
    await expect(page.locator("footer.site-footer")).toBeVisible();
  });
}

for (const path of protectedDataRoutes) {
  test(`${path} deep link reaches the subscription gate`, async ({ page }) => {
    await page.goto(path);

    await expect(page.locator("header.site-header")).toBeVisible();
    await expect(page.locator("footer.site-footer")).toBeVisible();
    await expect(
      page.getByRole("heading", { level: 1, name: /sign in to continue|couldn’t verify your access/i }),
    ).toBeVisible({ timeout: 15_000 });
    await expect(page.getByRole("button", { name: /choose plan|checkout|purchase/i })).toHaveCount(0);
  });
}

test("unknown routes render the not-found page", async ({ page }) => {
  await page.goto("/does-not-exist");
  await expect(page.getByRole("heading", { name: /not found/i })).toBeVisible();
});

test("subscription page only offers mobile-store handoffs", async ({ page }) => {
  await page.goto("/subscription");

  await expect(page.getByRole("link", { name: "Open on iPhone or iPad" })).toHaveAttribute(
    "href",
    /apps\.apple\.com/,
  );
  await expect(page.getByRole("link", { name: "Open on Android" })).toHaveAttribute(
    "href",
    /play\.google\.com/,
  );
  await expect(page.getByText(/created and managed only in the mobile app/i)).toBeVisible();
  await expect(page.getByRole("button", { name: /choose plan|checkout|purchase/i })).toHaveCount(0);
  await expect(page.getByText(/secure checkout/i)).toHaveCount(0);
});

test("subscribed users can filter and sort qualifying totals", async ({ page }) => {
  await mockSubscribedUser(page);
  await page.route("**/data/qualifying-totals", async (route) => {
    await route.fulfill(
      jsonResponse([
        {
          qualifying_total: 210,
          event_name: "Nationals",
          gender: "Women",
          age_category: "Senior",
          weight_class: "69kg",
        },
        {
          qualifying_total: 310,
          event_name: "Nationals",
          gender: "Men",
          age_category: "Senior",
          weight_class: "88kg",
        },
        {
          qualifying_total: 195,
          event_name: "American Open",
          gender: "Women",
          age_category: "Junior",
          weight_class: "63kg",
        },
        {
          qualifying_total: 340,
          event_name: "Nationals",
          gender: "Men",
          age_category: "Senior",
          weight_class: "110+kg",
        },
        {
          qualifying_total: 330,
          event_name: "Nationals",
          gender: "Men",
          age_category: "Senior",
          weight_class: "110kg",
        },
      ]),
    );
  });

  await page.goto("/qualifying-totals");
  await expect(page.getByRole("heading", { level: 1, name: "Qualifying totals" })).toBeVisible();
  expect(await page.getByLabel("Weight class").locator("option").allTextContents()).toEqual([
    "All classes",
    "63kg",
    "69kg",
    "88kg",
    "110kg",
    "110+kg",
  ]);
  await page.getByLabel("Gender").selectOption("Women");
  await expect(page.locator("tbody tr")).toHaveCount(2);
  await expect(page.locator("tbody")).not.toContainText("88kg");

  await page.getByLabel("Sort").selectOption("total_desc");
  await expect(page.locator("tbody tr").first()).toContainText("210");
  await expect(page.locator("tbody tr").last()).toContainText("195");
});

test("national rankings submit the expected query and rank totals descending", async ({ page }) => {
  await mockSubscribedUser(page);
  let requestedUrl = "";
  await page.route("**/data/nat-rankings-year?**", async (route) => {
    requestedUrl = route.request().url();
    await route.fulfill(
      jsonResponse([
        { name: "Second Athlete", total: 240, date: null },
        { name: "First Athlete", total: 260, date: "2026-03-01" },
      ]),
    );
  });

  await page.goto("/national-rankings");
  await page.getByLabel("Federation").selectOption("USAMW");
  await page.getByLabel("Gender").selectOption("Women");
  await page.getByLabel("Age group").selectOption("Masters 40");
  await page.getByLabel("Division").selectOption("Women's Masters (40-44) 69kg");
  await page.getByLabel("Year (optional)").fill("2026");
  await page.getByRole("button", { name: "View rankings" }).click();

  await expect(page.locator("tbody tr")).toHaveCount(2);
  await expect(page.locator("tbody tr").first()).toContainText("First Athlete");
  expect(requestedUrl).toContain("federation=USAMW");
  expect(requestedUrl).toContain("year=2026");
  expect(requestedUrl).toContain("age_category=Women%27s+Masters");
});

test("WSO records omit the organization column after an organization is selected", async ({ page }) => {
  await mockSubscribedUser(page);
  await page.route("**/data/wso", async (route) => {
    await route.fulfill(jsonResponse(["California North"]));
  });
  await page.route("**/data/wso/records?**", async (route) => {
    await route.fulfill(
      jsonResponse([
        {
          age_category: "Senior",
          cj_record: 189,
          gender: "Men",
          snatch_record: 158,
          total_record: 347,
          weight_class: "110+kg",
          wso: "California North",
        },
      ]),
    );
  });

  await page.goto("/wso-records");
  await page.getByLabel("Organization").selectOption("California North");

  await expect(page.locator("thead th")).toHaveCount(6);
  await expect(page.getByRole("columnheader", { name: "WSO" })).toHaveCount(0);
  await expect(page.locator("tbody tr").first().locator("td")).toHaveCount(6);
  await expect(page.locator("tbody")).not.toContainText("California North");
});

const completeResult = {
  federation: "USAW",
  meet: "Test Meet",
  date: "2026-06-20",
  name: "Test Athlete",
  age: "Senior",
  body_weight: 70.5,
  snatch1: 95,
  snatch2: 100,
  snatch3: -103,
  snatch_best: 100,
  cj1: 120,
  cj2: 125,
  cj3: 0,
  cj_best: 125,
  total: 225,
  adaptive: true,
};

const completedMeet = {
  federation: "USAW",
  end_date: "2026-06-20",
  name: "Test Meet",
  start_date: "2026-06-20",
  time_zone: "America/Los_Angeles",
  venue_city: "Oakland",
  venue_name: "Test Arena",
  venue_state: "CA",
  venue_street: "100 Main St",
  venue_zip: "94612",
  status: "completed",
  venue_map_pdf_url: null,
  venue_map_apple_url: "https://maps.apple.com/?q=Test+Arena",
};

test("athlete results show every competition field except federation", async ({ page }) => {
  await mockSubscribedUser(page);
  await page.route("**/search?**", async (route) => {
    await route.fulfill(jsonResponse({ matched_name: "Test Athlete", suggestions: [], results: [completeResult] }));
  });
  await page.goto("/results");
  await page.getByPlaceholder("Athlete name").fill("Test Athlete");
  await page.getByRole("button", { name: "Search" }).click();

  const headers = await page.getByRole("columnheader").allTextContents();
  expect(headers).toEqual(["Date", "Meet", "Division", "Bodyweight", "S1", "S2", "S3", "Best snatch", "C&J 1", "C&J 2", "C&J 3", "Best C&J", "Total", "Adaptive"]);
  expect(headers).not.toContain("Federation");
  await expect(page.locator("tbody tr")).toContainText("103×");
  await expect(page.locator("tbody tr")).toContainText("Yes");
});

test("meet center joins details, schedule, start list, and full results", async ({ page }) => {
  await mockSubscribedUser(page);
  await page.route("**/meets", (route) => route.fulfill(jsonResponse([])));
  await page.route("**/meets/completed", (route) => route.fulfill(jsonResponse([completedMeet])));
  await page.route("**/meets/schedule?**", (route) => route.fulfill(jsonResponse([{ date: "2026-06-20", meet: "Test Meet", platform: "Red", session_id: 1, start_time: "10:00", weigh_in_time: "08:00", weight_class: "69kg" }])));
  await page.route("**/meets/athletes-sessions?**", (route) => route.fulfill(jsonResponse([{ member_id: "1", name: "Test Athlete", age: 27, club: "Test Barbell", wso: "California North", gender: "Women", weight_class: "69kg", entry_total: 220, adaptive: false, session_number: 1, session_platform: "Red", date: "2026-06-20", start_time: "10:00", weigh_in_time: "08:00" }])));
  await page.route("**/lifting-results?**", (route) => route.fulfill(jsonResponse([completeResult])));
  await page.goto("/meet-center");
  await page.getByRole("combobox", { name: "Meet" }).selectOption("Test Meet");

  await expect(page.getByText("Test Arena", { exact: false })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Schedule" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Start list" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Full results" })).toBeVisible();
  await expect(page.locator("tbody").nth(1)).toContainText("Test Athlete");
  await expect(page.locator("tbody").nth(2)).toContainText("Test Athlete");
});

test("club and WSO dashboards expose meet performance metrics", async ({ page }) => {
  await mockSubscribedUser(page);
  await page.route("**/clubs", (route) => route.fulfill(jsonResponse(["Test Barbell"])));
  await page.route("**/clubs/athletes?**", (route) => route.fulfill(jsonResponse([{ meet: "Test Meet" }])));
  await page.route("**/clubs/meet-stats?**", (route) => route.fulfill(jsonResponse({ total_athletes: 1, gold_medals: 1, silver_medals: 0, bronze_medals: 0, total_prs: 1, perfect_6_for_6: 0, total_weight_lifted: 225, snatch_make_rate: 67, cj_make_rate: 67, combined_make_rate: 67, athlete_results: [{ name: "Test Athlete", weight_class: "69kg", snatch_best: 100, cj_best: 125, total: 225, body_weight: 70.5, medal: "Gold", snatch_medal: "Gold", cj_medal: "Gold", total_medal: "Gold", is_pr: true, perfect_lifts: false }] })));
  await page.goto("/club-dashboard");
  await page.getByLabel("Club").selectOption("Test Barbell");
  await page.getByRole("combobox", { name: "Meet" }).selectOption("Test Meet");
  await expect(page.getByText("Gold medals")).toBeVisible();
  await expect(page.locator("tbody")).toContainText("Test Athlete");

  await page.route("**/meets", (route) => route.fulfill(jsonResponse([])));
  await page.route("**/meets/completed", (route) => route.fulfill(jsonResponse([completedMeet])));
  await page.route("**/meets/athletes?**", (route) => route.fulfill(jsonResponse([{ member_id: "1", meet: "Test Meet", name: "Test Athlete", age: 27, club: "Test Barbell", wso: "California North", gender: "Women", weight_class: "69kg", entry_total: 220, adaptive: false, session_number: 1, session_platform: "Red" }])));
  await page.route("**/lifting-results?**", (route) => route.fulfill(jsonResponse([completeResult])));
  await page.goto("/wso-dashboard");
  await page.getByRole("combobox", { name: "Meet" }).selectOption("Test Meet");
  await page.getByLabel("WSO").selectOption("California North");
  await expect(page.getByText("WSO athletes")).toBeVisible();
  await expect(page.locator("tbody")).toContainText("225");
});

test("wrapped supports yearly athlete comparisons", async ({ page }) => {
  await mockSubscribedUser(page);
  await page.route("**/search?**", async (route) => {
    const name = new URL(route.request().url()).searchParams.get("query") ?? "";
    const result = { ...completeResult, name, total: name === "Second Athlete" ? 235 : 225 };
    await route.fulfill(jsonResponse({ matched_name: name, suggestions: [], results: [result] }));
  });
  await page.goto("/wrapped");
  await page.getByLabel("Athlete", { exact: true }).fill("Test Athlete");
  await page.getByLabel("Compare with (optional)").fill("Second Athlete");
  await page.getByLabel("Year").fill("2026");
  await page.getByRole("button", { name: "Build wrapped" }).click();

  await expect(page.getByRole("heading", { name: "2026 wrapped — Test Athlete" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Comparison", exact: true })).toBeVisible();
  await expect(page.getByRole("columnheader", { name: "Second Athlete" })).toBeVisible();
});

test("legal pages expose the declared privacy services and cross-link", async ({ page }) => {
  await page.goto("/privacy");
  await expect(page.getByText("Clerk, for authentication and account management.", { exact: true })).toBeVisible();
  await expect(page.getByText(/RevenueCat, Apple, and Google, for subscriptions/i)).toBeVisible();
  await expect(page.getByText("PostHog, for product analytics and diagnostics.", { exact: true })).toBeVisible();
  await expect(page.getByText(/only through the MeetCal mobile app/i)).toBeVisible();
  await expect(page.getByText(/does not currently provide a separate analytics preference control/i)).toBeVisible();
  await expect(page.getByText(/manage analytics through available browser or device controls/i)).toHaveCount(0);

  await page.goto("/terms");
  await page.getByRole("link", { name: "Privacy Policy" }).first().click();
  await expect(page).toHaveURL(/\/privacy$/);
});

test("routes expose specific titles, descriptions, canonicals, and indexing rules", async ({ page }) => {
  const expectations = [
    ["/", /Weightlifting Meet Schedules/, "index, follow"],
    ["/features", /MeetCal Features/, "index, follow"],
    ["/privacy", /Privacy Policy/, "index, follow"],
    ["/terms", /Terms of Use/, "index, follow"],
    ["/subscription", /Manage Your MeetCal Subscription/, "noindex, nofollow"],
    ["/qualifying-totals", /Weightlifting Qualifying Totals/, "noindex, nofollow"],
  ] as const;

  for (const [path, title, robots] of expectations) {
    await page.goto(path);
    await expect(page).toHaveTitle(title);
    await expect(page.locator('meta[name="description"]')).toHaveAttribute("content", /.{40,}/);
    await expect(page.locator('meta[name="robots"]')).toHaveAttribute("content", robots);
    await expect(page.locator('link[rel="canonical"]')).toHaveAttribute("href", `https://meetcal.app${path}`);
  }

  await page.goto("/does-not-exist");
  await expect(page).toHaveTitle(/Page Not Found/);
  await expect(page.locator('meta[name="robots"]')).toHaveAttribute("content", "noindex, nofollow");
});

test("footer uses valid, non-nested interactive links", async ({ page }) => {
  await page.goto("/");

  await expect(page.locator("a button, button a")).toHaveCount(0);
  await expect(page.getByRole("navigation", { name: "Legal" })).toBeVisible();
  await expect(page.locator('a[target="_blank"]:not([rel~="noopener"])')).toHaveCount(0);
});

test("public pages have no serious or critical accessibility violations", async ({ page }) => {
  for (const [path] of publicRoutes) {
    await page.goto(path);
    const results = await new AxeBuilder({ page })
      .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"])
      .analyze();
    const materialViolations = results.violations.filter(
      ({ impact }) => impact === "serious" || impact === "critical",
    );
    expect(materialViolations, `${path} accessibility violations`).toEqual([]);
  }
});

test("desktop navigation remains visible and complete", async ({ page, isMobile }) => {
  test.skip(isMobile, "Desktop-only navigation assertion");
  await page.goto("/");

  await expect(page.getByRole("navigation", { name: "Primary navigation" })).toBeVisible();
  await expect(page.locator("details.mobile-nav")).toBeHidden();
  await expect(page.getByRole("link", { name: "Competition Data" }).first()).toBeVisible();
});

test("mobile navigation opens and supports internal navigation", async ({ page, isMobile }) => {
  test.skip(!isMobile, "Mobile-only navigation assertion");
  await page.goto("/");

  await expect(page.getByRole("navigation", { name: "Primary navigation" })).toBeHidden();
  const menu = page.locator("details.mobile-nav");
  await menu.locator("summary").click();
  await expect(page.getByRole("navigation", { name: "Mobile navigation" })).toBeVisible();
  await page.getByRole("navigation", { name: "Mobile navigation" }).getByRole("link", { name: "Features" }).click();
  await expect(page).toHaveURL(/\/features$/);
  await expect(page.getByRole("heading", { level: 1, name: "Everything you need before the bar is loaded" })).toBeVisible();
  await expect(page.locator("details.mobile-nav")).not.toHaveAttribute("open", "");
});

test("mobile pages and key containers stay within the viewport", async ({ page, isMobile }) => {
  test.skip(!isMobile, "Mobile-only layout assertion");

  for (const [path] of publicRoutes) {
    await page.goto(path);
    const layout = await page.evaluate(() => {
      const viewport = document.documentElement.clientWidth;
      const selectors = [".site-header", ".hero-section", ".features-page", ".access-card", "main main", ".site-footer"];
      const boxes = selectors.flatMap((selector) =>
        [...document.querySelectorAll<HTMLElement>(selector)]
          .filter((element) => element.offsetParent !== null)
          .map((element) => ({ selector, ...element.getBoundingClientRect().toJSON() })),
      );
      return {
        viewport,
        scrollWidth: document.documentElement.scrollWidth,
        offenders: boxes.filter(({ left, right }) => left < -1 || right > viewport + 1),
      };
    });

    expect(layout.scrollWidth, `${path} page width`).toBeLessThanOrEqual(layout.viewport + 1);
    expect(layout.offenders, `${path} overflowing containers`).toEqual([]);
  }
});

test("mobile subscription actions are full-width touch targets", async ({ page, isMobile }) => {
  test.skip(!isMobile, "Mobile-only layout assertion");
  await page.goto("/subscription");

  const links = page.locator(".mobile-subscription-links .access-button");
  await expect(links).toHaveCount(2);
  const container = await page.locator(".mobile-subscription-links").boundingBox();
  for (const link of await links.all()) {
    const box = await link.boundingBox();
    expect(box?.height).toBeGreaterThanOrEqual(44);
    expect(box?.width).toBeGreaterThanOrEqual((container?.width ?? 0) - 1);
  }
});

test("authenticated mobile headers fit at narrow widths", async ({ page, isMobile }) => {
  test.skip(!isMobile, "Mobile-only authenticated layout assertion");
  await mockSubscribedUser(page);
  await page.route("**/data/qualifying-totals", async (route) => {
    await route.fulfill(jsonResponse([]));
  });

  await page.setViewportSize({ width: 320, height: 568 });
  await page.goto("/qualifying-totals");
  await expect(page.getByRole("heading", { name: "Qualifying totals" })).toBeVisible();
  await expect(page.locator(".clerk-user-button")).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(321);
});

test("mobile layouts tolerate landscape and enlarged text", async ({ page, isMobile }) => {
  test.skip(!isMobile, "Mobile-only responsive layout assertion");
  await page.setViewportSize({ width: 568, height: 320 });
  await page.goto("/features");
  await page.addStyleTag({ content: "html { font-size: 125%; }" });

  const dimensions = await page.evaluate(() => ({
    viewport: document.documentElement.clientWidth,
    scrollWidth: document.documentElement.scrollWidth,
  }));
  expect(dimensions.scrollWidth).toBeLessThanOrEqual(dimensions.viewport + 1);
  await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
});

test("competition loading and error states stay contained", async ({ page, isMobile }) => {
  test.skip(!isMobile, "Mobile-only data-state layout assertion");
  await mockSubscribedUser(page);
  await page.route("**/data/qualifying-totals", async (route) => {
    await new Promise((resolve) => setTimeout(resolve, 1_500));
    await route.fulfill({ status: 503, ...jsonResponse({ error: "unavailable" }) });
  });

  await page.goto("/qualifying-totals");
  await expect(page.locator(".data-table-skeleton")).toBeVisible();
  await expect(page.getByText(/could not load qualifying totals/i)).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(
    await page.evaluate(() => document.documentElement.clientWidth + 1),
  );
});
