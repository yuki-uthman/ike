// The Quotations pane on the Worker's feed (issue #8; D31).
//
// Driving port: a real browser (Playwright/Chromium) at a phone viewport,
// loading the page from the real Worker (tests/support/app.mjs), signed in.
// The Worker's Odoo is a LOCAL FAKE speaking JSON-2, and the quotations cron
// is fired through workerd's own scheduled port. Every fact is read through
// the DOM a phone would get.
//
// Claims: before the first build the pane says when quotations will appear,
// and shows them on a later refresh without a reload; once built, it lists
// the open quotations under one "Quotations" card and no "Unpaid orders"
// card; "Not interested", All and the day bars work as before.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { chromium } from 'playwright';

import { signedInContext, startApp } from './support/app.mjs';
import { ODOO_DB, ODOO_KEY, quotationRecords, startFakeOdooJson2 } from './support/fake-odoo-json2.mjs';

const PHONE = { width: 390, height: 844 };
const QUOTATIONS_CRON = '7-59/15 * * * *';
const NOT_READY = 'Quotations appear after the next refresh, within 15 minutes.';

async function waitFor(what, fn, timeout = 5000) {
  const deadline = Date.now() + timeout;
  let last;
  for (;;) {
    try { last = await fn(); if (last) return last; } catch (e) { last = e.message; }
    if (Date.now() > deadline) assert.fail(`timed out waiting for ${what} (last: ${last})`);
    await new Promise(r => setTimeout(r, 50));
  }
}

async function open(t) {
  const odoo = await startFakeOdooJson2(quotationRecords());
  const app = await startApp({ bindings: { ODOO_URL: odoo.url, ODOO_DB, ODOO_API_KEY: ODOO_KEY } });
  const browser = await chromium.launch();
  t.after(async () => { await browser.close(); await app.close(); await odoo.close(); });

  const context = await signedInContext(browser, app, { viewport: PHONE, hasTouch: true, isMobile: true });
  const page = await context.newPage();
  const crashes = [];
  page.on('pageerror', e => crashes.push(String(e)));
  const offOrigin = [];
  await context.route('**/*', route => {
    const url = route.request().url();
    if (url.startsWith(app.origin + '/')) return route.continue();
    offOrigin.push(url);
    const body = url.endsWith('.json') ? '{"days":[]}' : '';
    return route.fulfill({ status: 200, contentType: 'application/json', body });
  });
  await page.goto(app.origin + '/', { waitUntil: 'load' });
  await page.click('#tab-quotes');

  const build = async () => {
    const res = await fetch(`${app.origin}/cdn-cgi/handler/scheduled?cron=${encodeURIComponent(QUOTATIONS_CRON)}`);
    await res.text();
  };
  return { page, build, crashes, offOrigin };
}

const message = page => page.evaluate(() => {
  const el = document.querySelector('#pane-quotes .state-msg');
  return el ? el.textContent : null;
});
const rowRefs = page => page.$$eval('#q-content .row', rows => rows.map(r => r.getAttribute('data-ref')));

test('before_the_first_build_the_pane_says_when_and_a_refresh_brings_the_quotations', async t => {
  const { page, build, crashes, offOrigin } = await open(t);

  await waitFor('the not-ready message', async () => (await message(page)) === NOT_READY);
  assert.equal(await page.isVisible('#q-hero'), false, 'nothing else of the pane shows meanwhile');

  await build();
  // The pane's own refresh (on return to the tab), not a reload.
  await page.evaluate(() => document.dispatchEvent(new Event('visibilitychange')));
  await waitFor('the quotations', async () => (await message(page)) === null
    && await page.isVisible('#q-hero'));

  assert.ok(!offOrigin.some(u => u.includes('quotations.json')), 'ike-data\'s quotations.json is not read');
  assert.deepEqual(crashes, []);
});

test('the_pane_lists_open_quotations_only_and_not_interested_still_works', async t => {
  const { page, build, crashes } = await open(t);
  await build();
  await page.reload({ waitUntil: 'load' });
  await page.click('#tab-quotes');
  await waitFor('the hero', () => page.isVisible('#q-hero'));

  // All days: every open quotation, biggest first.
  await page.click('#q-allpill');
  await waitFor('three rows', async () => (await rowRefs(page)).length === 3);
  assert.deepEqual(await rowRefs(page), ['S00101', 'S00103', 'S00104']);
  assert.equal(await page.textContent('#q-heroValue'), '1,351');
  assert.equal(await page.textContent('#q-subline'), '3 records still owing');

  const cards = await page.$$eval('#q-content .card .kind', els => els.map(e => e.textContent.trim()));
  assert.deepEqual(cards, ['Quotations'], 'one card: there is no Unpaid orders card');
  assert.ok(!(await page.textContent('#pane-quotes')).includes('Unpaid'));
  assert.ok(await page.isVisible('#q-content .row[data-ref="S00101"] .tag.part'), 'part paid at the till');

  // One day: the bar for the 19th holds S00103 only.
  await page.click('#q-chartPlot .bar[data-day="2026-10-19"]');
  await waitFor('the 19th', async () => (await rowRefs(page)).join() === 'S00103');
  await page.click('#q-allpill');
  await waitFor('all again', async () => (await rowRefs(page)).length === 3);

  // Not interested hides it, and the counter at the bottom brings it back.
  await page.click('#q-content .row[data-ref="S00101"]');
  await page.click('#q-content [data-drop="S00101"]');
  await waitFor('S00101 hidden', async () => (await rowRefs(page)).join() === 'S00103,S00104');
  assert.equal(await page.textContent('#q-heroValue'), '751');
  await page.click('#q-binToggle');
  await page.click('#q-bin [data-restore="S00101"]');
  await waitFor('S00101 back', async () => (await rowRefs(page)).length === 3);

  assert.deepEqual(crashes, []);
});
