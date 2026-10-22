// Oracle for "V3 Pane on real data".
//
// Authority: docs/product/brief.md#Decisions (D2, D7, D9, D10, D11, D12, D13,
// D14, D16) and #Outcomes, read through the DESIGN section "V3 Pane on real
// data". The brief is cited, never opened by this file.
//
// Driving port: a real browser (Playwright/Chromium) at the design's phone
// viewport, loading the page from the real Worker that serves it since D27
// (tests/support/app.mjs). Nothing is imported from the page; every fact below
// is read through the DOM a phone would get, or through the service's own HTTP
// contract.
//
// The service is NOT a stub: it is the real worker/src/index.mjs running on a
// real workerd (miniflare) over a real socket with a real D1, because the whole
// point of this observation is that the pane speaks to the actual contract.
//
// ike-data's expenses.json is answered by Playwright routing with a payload in
// the shape measured from the live feed, so the oracle does not depend on the
// live internet. An unreachable service is made by refusing the page's calls to
// its own /expenses/ area at the network edge — a same-origin page cannot be
// pointed at a dead port any more.
//
// Every handle below is public: the pane id #pane-expenses and the e- prefixed
// ids already in the page's markup, the FAB's aria-label "Add expense", the
// field labels "Amount" / "Category", the "Post expense" button, the sign-in
// screen's "Username" / "Password" / "Sign in", D12's chip words, and the
// service's own JSON members.
//
// Narrowed by D27 (2026-10-22): the wrong-PIN arm became the signed-out save
// (D28: the sheet keeps what was typed and the SAME entry is resent after
// sign-in); "no service configured" is gone, because the page always calls its
// own origin; and "?api= is loopback-only" is tests/odoo-layer-go-live's now.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { chromium } from 'playwright';

import { PASSWORD, USERNAME, signedInContext, startApp } from './support/app.mjs';

// The design screens are drawn at 390x844 — "on a phone".
const PHONE = { width: 390, height: 844 };

const FEED_URL =
  'https://raw.githubusercontent.com/yuki-uthman/ike-data/main/data/expenses.json';

// The off-origin requests the page may make: two font hosts and ike-data's three
// feeds. On its own origin it may ask for its files, /auth/ and /expenses/.
// Anything else is off limits.
const PRE_EXISTING = [
  'https://fonts.googleapis.com/',
  'https://fonts.gstatic.com/',
  'https://raw.githubusercontent.com/yuki-uthman/ike-data/main/data/sales.json',
  'https://raw.githubusercontent.com/yuki-uthman/ike-data/main/data/quotations.json',
  FEED_URL
];

const EMPTY_FEEDS = {
  'sales.json': '{"days":[]}',
  'quotations.json': '{"records":[],"days":[]}'
};

/** D16 / decision 21: the Maldives (UTC+5) date, the page's TODAY. */
function maldivesToday(at = Date.now()) {
  return new Date(at + 5 * 3600 * 1000).toISOString().slice(0, 10);
}

// Two real days of the feed, copied verbatim from the live document, plus the
// same figures carried on TODAY so the figures region has something to render
// for the day the pane opens on.
const DAY_EMPTY = {
  date: '2026-09-13',
  generatedAt: '2026-09-14T10:54:15Z',
  confirmed: { total: 0, count: 0 },
  pending: { total: 0, count: 0 },
  categories: []
};
const DAY_FULL_CATEGORIES = [
  { name: 'Vehicle Maintenance', count: 1, total: 130.0 },
  { name: 'Fuel / Petrol', count: 1, total: 50.0 },
  { name: 'Expenses', count: 1, total: 18.0 },
  { name: 'Gate Pass (Boat Delivery)', count: 2, total: 10.0 }
];
const DAY_FULL = {
  date: '2026-09-14',
  generatedAt: '2026-09-14T18:45:43Z',
  confirmed: { total: 208.0, count: 5 },
  pending: { total: 100030.0, count: 5 },
  categories: DAY_FULL_CATEGORIES
};

function feedDocument(days) {
  return JSON.stringify({ company: 'MRH Investment', currency: 'MVR', days });
}

/** The feed as it ordinarily is: ascending, carrying TODAY last. */
function feedWithToday(today) {
  return feedDocument([
    DAY_EMPTY,
    DAY_FULL,
    { ...DAY_FULL, date: today, generatedAt: today + 'T03:01:23Z' }
  ]);
}

/** The ordinary daily state between Maldives midnight and the next rebuild. */
function feedWithoutToday() {
  return feedDocument([DAY_EMPTY, DAY_FULL]);
}

const OWN_PATHS = /^\/(index\.html|sw\.js|manifest\.webmanifest|icons\/[^/]+|auth\/.*|expenses\/.*)?$/;

/**
 * Open the page from the app, signed in, with every request accounted for.
 * `control.down` makes every call to /expenses/ fail at the network edge — the
 * page's view of an unreachable service.
 */
async function openPage(browser, app, { feed, colorScheme = 'light' }) {
  const offLimits = [];
  const requested = [];
  const control = { down: false };
  const context = await signedInContext(browser, app, {
    viewport: PHONE, colorScheme, hasTouch: true, isMobile: true,
    deviceScaleFactor: 3
  });
  const page = await context.newPage();
  const crashes = [];
  page.on('pageerror', e => crashes.push(String(e)));

  await context.route('**/*', async route => {
    const url = route.request().url();
    requested.push(url);
    if (url.startsWith(app.origin)) {
      const at = new URL(url).pathname;
      if (!OWN_PATHS.test(at)) offLimits.push(url);
      if (control.down && at.startsWith('/expenses/')) return route.abort('connectionrefused');
      return route.continue();
    }
    if (url.startsWith(FEED_URL)) {
      return route.fulfill({ status: 200, contentType: 'application/json', body: feed });
    }
    const other = Object.keys(EMPTY_FEEDS).find(f => url.split('?')[0].endsWith(f));
    if (other) {
      return route.fulfill({
        status: 200, contentType: 'application/json', body: EMPTY_FEEDS[other]
      });
    }
    if (!PRE_EXISTING.some(p => url.startsWith(p))) offLimits.push(url);
    return route.fulfill({ status: 200, contentType: 'text/plain', body: '' });
  });

  await page.goto(`${app.origin}/`, { waitUntil: 'load' });
  return { page, context, offLimits, requested, crashes, control };
}

/** Open the pane and the Add sheet, and hand back its public handles. */
async function openSheet(page) {
  const expensesTab = page.locator('#tabs [role="tab"]').filter({ hasText: 'Expenses' });
  await expensesTab.click();
  await waitFor('the Expenses tab to be selected',
    async () => await expensesTab.getAttribute('aria-selected') === 'true');
  await page.locator('#pane-expenses [aria-label="Add expense"]').click();
  const handles = {
    amount: page.getByLabel('Amount'),
    category: page.getByLabel('Category'),
    post: page.getByRole('button', { name: 'Post expense' }),
    note: page.locator('#e-note')
  };
  await waitFor('the Add sheet to open', async () => await handles.post.isVisible());
  return handles;
}

async function waitFor(what, fn, timeout = 15000) {
  const deadline = Date.now() + timeout;
  let last;
  for (;;) {
    try { last = await fn(); if (last) return last; } catch (e) { last = e.message; }
    if (Date.now() > deadline) assert.fail(`timed out waiting for ${what} (last: ${last})`);
    await new Promise(r => setTimeout(r, 50));
  }
}

const CHIPS = ['Waiting to send', 'Not sent', 'Draft', 'Approved', 'Refused'];

/**
 * Read the entries card the way a reader does: every block inside #e-entries
 * that carries exactly one of D12's five chip words is one entry. No class of
 * the implementation is assumed.
 */
function readEntries(page, chips = CHIPS) {
  return page.evaluate(chipWords => {
    const card = document.getElementById('e-entries');
    if (!card) return [];
    const norm = s => (s || '').replace(/\s+/g, ' ').trim();
    const fieldText = el => {
      const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT);
      const parts = [];
      for (let n = walker.nextNode(); n; n = walker.nextNode()) {
        const t = norm(n.nodeValue);
        if (t) parts.push(t);
      }
      return norm(parts.join(' '));
    };
    const chipsIn = el => chipWords.filter(w => norm(el.textContent).includes(w)).length;
    const chipEls = Array.from(card.querySelectorAll('*')).filter(el =>
      chipWords.includes(norm(el.textContent)) &&
      !Array.from(el.children).some(c => chipWords.includes(norm(c.textContent))));
    return chipEls.map(chip => {
      let row = chip;
      while (row.parentElement && row.parentElement !== card
             && chipsIn(row.parentElement) === 1) {
        row = row.parentElement;
      }
      return {
        chip: norm(chip.textContent),
        text: fieldText(row),
        hasRetryButton: Array.from(row.querySelectorAll('button'))
          .some(b => norm(b.textContent) === 'Retry')
      };
    });
  }, chips);
}

const flat = s => (s || '').replace(/\s+/g, ' ').trim();

async function regionText(page, id) {
  return flat(await page.locator('#' + id).innerText());
}

test('pane_saves_through_the_service_and_shows_only_confirmed_entries', async t => {
  const TODAY = maldivesToday();

  const app = await startApp();
  const cookie = await app.signIn();
  const service = async pathAndQuery => {
    const r = await app.call('GET', pathAndQuery, { cookie });
    return { status: r.res.status, body: r.json };
  };
  const browser = await chromium.launch();

  t.after(async () => {
    await browser.close();
    await app.close();
  });

  // ================= 1. The service is the only author of the entry list ======
  const { page, offLimits, requested, crashes } = await openPage(browser, app, {
    feed: feedWithToday(TODAY)
  });

  await page.locator('#tabs [role="tab"]').filter({ hasText: 'Expenses' }).click();

  // The day's figures come from ike-data's feed, rendered as the feed gives them.
  await waitFor("the day's confirmed total to come from the feed",
    async () => flat(await page.locator('#e-heroTotal').innerText()) === '208');
  assert.match(await regionText(page, 'e-heroCount'), /\b5 entries\b/,
    "the hero's count is the feed day's confirmed.count");
  assert.equal(await regionText(page, 'e-pending'),
    'Pending MVR 100,030 · 5 entries not counted yet',
    'the pending line is the feed day\'s pending total and count, and nothing else');

  const catsText = await regionText(page, 'e-cats');
  assert.deepEqual(
    await page.locator('#e-cats .catname').allTextContents().then(x => x.map(flat)),
    DAY_FULL_CATEGORIES.map(c => c.name),
    'the category rows are the feed day\'s categories, in the order given, '
    + "including the generic 'Expenses' the read side must not hide");
  for (const c of DAY_FULL_CATEGORIES) {
    assert.ok(catsText.includes(String(c.total).replace(/\.0$/, '')),
      `the category row for ${c.name} carries its feed total`);
  }

  // The day pills are the feed's days, each addressable, oldest left and today
  // at the right end, as on Sales and Quotations.
  assert.deepEqual(
    await page.locator('#e-pills [data-day]').evaluateAll(
      els => els.map(e => e.getAttribute('data-day'))),
    [DAY_EMPTY.date, DAY_FULL.date, TODAY],
    'every feed day is a selectable pill, oldest left, today rightmost');

  // The entries card is the service's rows and nothing else: the store is empty.
  const emptyDay = await service(`/expenses/entries?date=${TODAY}`);
  assert.equal(emptyDay.status, 200);
  assert.deepEqual(emptyDay.body.entries, [], 'the service holds nothing for today yet');
  assert.deepEqual(await readEntries(page), [],
    'the pane shows no entry when the service has none');

  // The sheet's select is the service's category list, in the service's order.
  const cats = await service('/expenses/categories');
  assert.equal(cats.status, 200);
  assert.equal(cats.body.categories.length, 21, 'D7: the service serves 21 categories');
  const sheet = await openSheet(page);
  assert.deepEqual(
    (await sheet.category.locator('option').allTextContents()).map(flat),
    cats.body.categories,
    'the select carries exactly the service\'s categories, in the service\'s order');

  // ================= 2. Signed out mid-save: the entry waits, then goes ONCE ==
  // The phone's session is ended behind its back (the owner's `signout`). The
  // press meets a 401: the sign-in screen comes over the app, the sheet keeps
  // what was typed underneath, and after sign-in the SAME entry is sent again
  // (D28) — one row, never two.
  await sheet.amount.fill('137.50');
  await sheet.category.selectOption({ label: 'Fuel / Petrol' });
  await app.db.prepare('DELETE FROM session').run();
  await sheet.post.click();

  const signin = page.locator('#signin');
  await waitFor('the sign-in screen over the app', async () =>
    await signin.isVisible() && await page.getByLabel('Password').isVisible());
  assert.deepEqual(
    (await service(`/expenses/entries?date=${TODAY}`)).status, 401,
    'the old session is gone for every caller');
  assert.equal(await sheet.amount.inputValue(), '137.50',
    'the typed amount is still in the sheet under the sign-in screen');
  assert.equal(await sheet.category.inputValue(), 'Fuel / Petrol',
    'and the chosen category');
  assert.equal(await app.db.prepare('SELECT COUNT(*) AS n FROM entry').first()
    .then(r => r.n), 0, 'the signed-out press stored nothing');

  await page.getByLabel('Username').fill(USERNAME);
  await page.getByLabel('Password').fill(PASSWORD);
  const started = Date.now();
  await page.getByRole('button', { name: 'Sign in' }).click();
  // Read the layers themselves: while the sign-in screen covers the app, the
  // sheet's controls are hidden from the accessibility tree, so a role locator
  // on them would read "not visible" at once and prove nothing.
  await waitFor('the sign-in screen to go once signed in', async () => await signin.isHidden());
  await waitFor('the sheet to close on the resent save\'s 200',
    async () => await page.locator('#e-sheet').isHidden());
  const elapsed = Date.now() - started;
  assert.ok(elapsed < 30000,
    `the save closes the sheet inside the 30-second budget (took ${elapsed} ms)`);

  const fresh = await app.signIn();
  const serviceNow = async pathAndQuery => {
    const r = await app.call('GET', pathAndQuery, { cookie: fresh });
    return { status: r.res.status, body: r.json };
  };

  // ================= 3. What the service holds is what the page shows =======
  const saved = await waitFor('the saved entry to appear in the day\'s list', async () => {
    const rows = await readEntries(page);
    return rows.length ? rows : null;
  });
  const day = await serviceNow(`/expenses/entries?date=${TODAY}`);
  assert.equal(day.body.entries.length, 1, 'the service stored exactly one entry');
  const entry = day.body.entries[0];
  assert.equal(entry.status, 'Waiting to send',
    "the save-first status word, and the chip the pane must show");
  assert.equal(saved.length, 1, 'the page shows exactly the one row the service holds');
  assert.equal(saved[0].chip, entry.status,
    'the chip is the service\'s own word for the entry');
  assert.ok(saved[0].text.includes(entry.category),
    'the row carries the category the service recorded');
  assert.ok(saved[0].text.includes(entry.amount_mvr)
    || saved[0].text.includes(entry.amount_mvr.replace(/\.00$/, '')),
    `the row carries the amount as the service formatted it (${entry.amount_mvr})`);
  assert.ok(!saved[0].hasRetryButton,
    'a waiting entry offers no Retry: the contract carries no next-retry time');
  assert.equal(entry.amount_mvr, '137.50', 'the amount reached the store as typed MVR');

  // Both presses carried ONE minted id: the resend is the same entry.
  const posts = requested.filter(u => u === app.origin + '/expenses/entries');
  assert.ok(posts.length >= 2, 'the refused press and the resend both went to the service');
  assert.match(entry.client_entry_id, /^[0-9a-f-]{36}$/,
    'the entry carries the id the page minted');
  const stored = await page.evaluate(() => ({
    local: Object.entries(localStorage).map(([k, v]) => k + '=' + v).join('|'),
    session: Object.entries(sessionStorage).map(([k, v]) => k + '=' + v).join('|'),
    cookie: document.cookie
  }));
  assert.ok(!stored.local.includes(PASSWORD) && !stored.session.includes(PASSWORD),
    'the password is never stored in the browser');
  assert.ok(!stored.cookie.includes('ike_session'),
    'the session cookie is HttpOnly: no script on the page can read it');

  // A reload can add nothing: the list is re-read from the service, and the
  // phone is still signed in.
  await page.reload({ waitUntil: 'load' });
  await page.locator('#tabs [role="tab"]').filter({ hasText: 'Expenses' }).click();
  const afterReload = await waitFor('the list after a reload', async () => {
    const rows = await readEntries(page);
    return rows.length ? rows : null;
  });
  assert.equal(afterReload.length, 1,
    'a reload shows the one entry the service holds, not one the page remembered');
  assert.ok(await signin.isHidden(), 'a reload keeps the phone signed in');

  assert.deepEqual(offLimits, [],
    `the page contacted nothing beyond its feeds and its own areas: ${offLimits.join(', ')}`);
  assert.deepEqual(crashes, [], `no script error: ${crashes.join(' | ')}`);

  // ================= 4. The two regions fail independently ==================
  // The feed has no record for TODAY yet (the ordinary state between Maldives
  // midnight and the next rebuild): the figures say so, and the entries card,
  // which reads the service, keeps showing the day's row.
  const noFeedDay = await openPage(browser, app, { feed: feedWithoutToday() });
  await noFeedDay.page.locator('#tabs [role="tab"]').filter({ hasText: 'Expenses' }).click();
  await waitFor('the figures to name the missing day', async () => {
    const text = flat(await noFeedDay.page.locator('#pane-expenses').innerText());
    return text.includes("Could not load the day's totals")
      && text.includes(TODAY)
      && text.includes('Try refreshing in a minute.');
  });
  const stillListed = await waitFor('the entries card to read the service', async () => {
    const rows = await readEntries(noFeedDay.page);
    return rows.length ? rows : null;
  });
  assert.equal(stillListed.length, 1,
    'a feed without the day does not blank the service-backed entries card');
  assert.ok(await noFeedDay.page.locator('#pane-expenses [aria-label="Add expense"]').isVisible(),
    'the pane is never replaced by a message: the FAB stays reachable');
  assert.deepEqual(noFeedDay.crashes, [], noFeedDay.crashes.join(' | '));

  // ================= 5. An unreachable service =============================
  const down = await openPage(browser, app, { feed: feedWithToday(TODAY) });
  down.control.down = true;
  await down.page.reload({ waitUntil: 'load' });
  await down.page.locator('#tabs [role="tab"]').filter({ hasText: 'Expenses' }).click();
  await waitFor('the entries card to say it could not load', async () => {
    const text = await regionText(down.page, 'e-entries');
    return text.includes("Could not load today's entries")
      && text.includes('Nothing is shown as saved.');
  });
  assert.deepEqual(await readEntries(down.page), [],
    'an unreadable list shows no row at all');
  assert.equal(flat(await down.page.locator('#e-heroTotal').innerText()), '208',
    'the figures region is unaffected by the service being down');

  // The Add sheet when the category list is not readable: the select stays the
  // service's list and nothing else, so there is nothing to choose and the sheet
  // refuses to pretend otherwise. Post is asserted disabled, never clicked — a
  // real click on a disabled button is not delivered at all.
  const downSheet = await openSheet(down.page);
  await waitFor('the failed-categories note', async () =>
    flat(await downSheet.note.innerText())
      === 'Could not load the categories (service unreachable). Try again in a minute.');
  assert.deepEqual(
    await downSheet.category.locator('option').allTextContents(), [],
    'no copy of the 21 names lives in the page: with no readable list there is '
    + 'no option to choose');
  assert.equal(await downSheet.post.isDisabled(), true,
    'Post is disabled, so an entry whose category the service never offered '
    + 'cannot be sent');

  // Typed fields are never cleared or rewritten by that rendering.
  await downSheet.amount.fill('80');
  await new Promise(r => setTimeout(r, 1000));
  assert.equal(await downSheet.amount.inputValue(), '80', 'the typed amount is kept');
  assert.equal(await downSheet.post.isDisabled(), true, 'Post is still disabled');
  assert.ok(await down.page.locator('#pane-expenses [aria-label="Add expense"]').isVisible(),
    'the FAB and the sheet stay reachable while the service is down');
  assert.equal(flat(await down.page.locator('#e-heroTotal').innerText()), '208',
    'and the figures region kept reading the feed');
  assert.deepEqual(await readEntries(down.page), [],
    'nothing the service did not confirm is ever shown as saved');
  assert.deepEqual(down.crashes, [], down.crashes.join(' | '));

  // ========== 5c. A category list re-read while the sheet is open ===========
  // Opening the sheet re-reads /expenses/categories, so a service that comes back fills
  // the select with no reload. That answer can land AFTER the person has chosen
  // (the open-sheet re-read, the 5-minute refresh, a return to the tab). The
  // choice must survive it: "keeps the typed entry in the sheet" covers the
  // category as much as the amount. The re-read is held at the network edge
  // until the choice is made, so the order is fixed, not raced.
  const held = await openPage(browser, app, { feed: feedWithToday(TODAY) });
  t.after(async () => { await held.context.close().catch(() => {}); });
  await waitFor('the first category read to fill the select', async () =>
    (await held.page.getByLabel('Category').locator('option').count()) === 21);
  let release;
  const gate = new Promise(r => { release = r; });
  let heldReads = 0;
  await held.page.route(app.origin + '/expenses/categories', async route => {
    heldReads++;
    await gate;
    await route.continue();
  });
  const heldSheet = await openSheet(held.page);
  await waitFor('the open-sheet re-read to be in flight', async () => heldReads >= 1);
  await heldSheet.amount.fill('80');
  await heldSheet.category.selectOption({ label: 'Meals' });
  const reread = held.page.waitForResponse(r => r.url().startsWith(app.origin + '/expenses/categories'));
  release();
  await reread;
  await held.page.evaluate(() => new Promise(r => setTimeout(r, 500)));
  assert.equal(await heldSheet.category.locator('option').count(), 21,
    'the re-read list is the service\'s 21 names');
  assert.equal(await heldSheet.category.inputValue(), 'Meals',
    'a category list that arrives after the choice keeps the chosen category');
  assert.equal(await heldSheet.amount.inputValue(), '80', 'and the typed amount');
  assert.equal(await heldSheet.post.isDisabled(), false, 'and Post stays enabled');
  assert.deepEqual(held.crashes, [], held.crashes.join(' | '));

  // ========== 5b. The service dies AFTER the select loaded ==================
  // This is the stimulus the observation names with "an unreachable service
  // keeps the typed entry in the sheet with a plain message": the select is
  // filled with the service's own 21 names, then the service becomes
  // unreachable, and only then is Post pressed.
  const dying = await openPage(browser, app, { feed: feedWithToday(TODAY) });
  await dying.page.locator('#tabs [role="tab"]').filter({ hasText: 'Expenses' }).click();
  const dyingSheet = await openSheet(dying.page);
  await waitFor('the select to fill from the live service', async () =>
    (await dyingSheet.category.locator('option').count()) === 21);
  assert.equal(await dyingSheet.post.isDisabled(), false,
    'with the list read, Post is enabled');
  await dyingSheet.amount.fill('80');
  await dyingSheet.category.selectOption({ label: 'Meals' });
  const rowsBefore = await waitFor('the day\'s row on the dying page', async () => {
    const rows = await readEntries(dying.page);
    return rows.length ? rows : null;
  });

  dying.control.down = true;          // the service becomes unreachable

  await dyingSheet.post.click();
  await waitFor('the unreachable message',
    async () => flat(await dyingSheet.note.innerText())
      === 'Could not reach the expenses service. Your entry is still here — try again.');
  assert.ok(await dyingSheet.post.isVisible(), 'the sheet stays open');
  assert.equal(await dyingSheet.amount.inputValue(), '80', 'the typed entry is still there');
  assert.equal(await dyingSheet.category.inputValue(), 'Meals',
    'including the chosen category');
  assert.deepEqual(await readEntries(dying.page), rowsBefore,
    'no row is appended on any non-200 path');
  assert.deepEqual(dying.crashes, [], dying.crashes.join(' | '));

  // ================= 8. Nothing reached the service it should not have =====
  const finalDay = await serviceNow(`/expenses/entries?date=${TODAY}`);
  assert.equal(finalDay.body.entries.length, 1,
    'across every path, exactly the one confirmed entry exists');
});
