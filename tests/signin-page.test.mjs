// Oracle for the sign-in screen and Sign out, as a reader meets them (issue #7).
//
// Authority: docs/product/brief.md#Decisions D27, D28 (the owner's "ok" to the
// screen and the Sign out placement, recorded in yuki-uthman/ike#7).
//
// Driving port: a real browser (Playwright/Chromium) at the design's phone
// viewport, loading the page from the real Worker that serves it
// (tests/support/app.mjs). Every handle is public: the field labels
// "Username" / "Password", the "Sign in" button, the visible words, and the
// "Signed in as … · Sign out" line under the Expenses legend. The signed-out
// save and its resend are tests/expenses-real-data.test.mjs's.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { chromium } from 'playwright';

import { PASSWORD, USERNAME, startApp } from './support/app.mjs';

const PHONE = { width: 390, height: 844 };
const flat = s => (s || '').replace(/\s+/g, ' ').trim();

async function waitFor(what, fn, timeout = 15000) {
  const deadline = Date.now() + timeout;
  let last;
  for (;;) {
    try { last = await fn(); if (last) return last; } catch (e) { last = e.message; }
    if (Date.now() > deadline) assert.fail(`timed out waiting for ${what} (last: ${last})`);
    await new Promise(r => setTimeout(r, 50));
  }
}

/** A fresh phone: no cookie, the off-origin feeds answered empty. */
async function freshPhone(browser, app, colorScheme = 'light') {
  const context = await browser.newContext({ viewport: PHONE, colorScheme, isMobile: true });
  await context.route(url => !String(url).startsWith(app.origin), route =>
    route.fulfill({ status: 200, contentType: 'application/json', body: '{"days":[],"records":[]}' }));
  const page = await context.newPage();
  const crashes = [];
  page.on('pageerror', e => crashes.push(String(e)));
  await page.goto(app.origin + '/', { waitUntil: 'load' });
  return { context, page, crashes };
}

test('a_signed_out_phone_sees_only_the_sign_in_screen_and_signs_in_and_out', async t => {
  const app = await startApp();
  const browser = await chromium.launch();
  t.after(async () => { await browser.close(); await app.close(); });

  const { page, crashes } = await freshPhone(browser, app);
  const username = page.getByLabel('Username');
  const password = page.getByLabel('Password');
  const go = page.getByRole('button', { name: 'Sign in' });
  await waitFor('the sign-in form', async () => await password.isVisible());

  // ---- Only the sign-in screen: the app behind it is unreachable ----
  const screen = await page.evaluate(() => {
    const s = document.getElementById('signin');
    const r = s.getBoundingClientRect();
    const behind = Array.from(document.getElementById('app').children).filter(el => el !== s);
    return {
      covers: r.left <= 0 && r.top <= 0 && r.right >= innerWidth && r.bottom >= innerHeight,
      opaque: getComputedStyle(s).backgroundColor,
      inert: behind.every(el => el.inert && el.getAttribute('aria-hidden') === 'true'),
      text: s.innerText
    };
  });
  assert.ok(screen.covers, 'the sign-in screen covers the whole phone');
  assert.ok(!/rgba\(.*, 0\)|transparent/.test(screen.opaque), 'and is opaque: no figure shows behind it');
  assert.ok(screen.inert, 'everything behind it is inert and hidden from VoiceOver');
  assert.match(flat(screen.text), /MRH INVESTMENT/i, 'the eyebrow reads MRH Investment');
  assert.match(flat(screen.text), /\bIke\b/, 'the title reads Ike');
  assert.equal(await page.locator('#tabs').getByRole('tab').count(), 0,
    'not one tab of the app is reachable through the accessibility tree');

  // ---- Chrome can offer to save the password ----
  assert.equal(await username.getAttribute('autocomplete'), 'username');
  assert.equal(await password.getAttribute('autocomplete'), 'current-password');
  assert.equal(await password.getAttribute('type'), 'password');
  assert.equal(await page.locator('#signin form').count(), 1, 'the fields sit in a real form');

  // ---- A wrong password: one red line, nothing else ----
  const error = page.locator('#signin-error');
  assert.equal(flat(await error.innerText()), '', 'no error line before a failure');
  await username.fill(USERNAME);
  await password.fill('wrong-wrong-wrong');
  await go.click();
  await waitFor('the wrong-password line', async () =>
    flat(await error.innerText()) === 'Wrong username or password.');
  const red = await error.evaluate(el => getComputedStyle(el).color);
  const [r, g, b] = red.match(/\d+/g).map(Number);
  assert.ok(r > g && r > b, `the line is red (${red})`);
  assert.equal(await password.inputValue(), '', 'the wrong password is cleared');
  assert.equal(await username.inputValue(), USERNAME, 'the username is kept');

  // ---- A service that cannot answer is not called a wrong password ----
  await page.route(app.origin + '/auth/signin', route =>
    route.fulfill({ status: 503, contentType: 'application/json',
      body: '{"saved":false,"error":"not_read"}' }), { times: 1 });
  await password.fill(PASSWORD);
  await go.click();
  await waitFor('the could-not-sign-in line', async () =>
    /^Could not sign in just now \(503\)\. Try again\.$/.test(flat(await error.innerText())));

  // ---- The right one: the app, and who is signed in ----
  await password.fill(PASSWORD);
  await go.click();
  await waitFor('the app behind the sign-in screen', async () =>
    await page.locator('#signin').isHidden());
  assert.equal(await page.locator('#tabs').getByRole('tab').count(), 3, 'the three tabs are back');
  await page.locator('#tabs [role="tab"]').filter({ hasText: 'Expenses' }).click();
  const who = page.locator('#e-signout');
  await waitFor('the Signed in as line', async () => await who.isVisible());
  assert.equal(flat(await who.innerText()), `Signed in as ${USERNAME} · Sign out`,
    'the bottom of Expenses names the account and offers Sign out (D28)');
  const legendBefore = await page.evaluate(() => {
    const legend = document.querySelector('#pane-expenses .legend');
    const line = document.getElementById('e-signout');
    return !!(legend.compareDocumentPosition(line) & Node.DOCUMENT_POSITION_FOLLOWING);
  });
  assert.ok(legendBefore, 'the line sits under the Waiting › Draft › Approved legend');

  // ---- A reopened app is still signed in ----
  await page.reload({ waitUntil: 'load' });
  await waitFor('the app on a reopen', async () => (await page.locator('#tabs').getByRole('tab').count()) === 3);
  assert.ok(await page.locator('#signin').isHidden(), 'no sign-in on a reopen');

  // ---- Sign out: no question asked, this phone only, back to the screen ----
  const otherPhone = await app.signIn();
  await page.locator('#tabs [role="tab"]').filter({ hasText: 'Expenses' }).click();
  let dialogs = 0;
  page.on('dialog', d => { dialogs += 1; d.dismiss(); });
  await page.getByRole('button', { name: 'Sign out' }).click();
  await waitFor('the sign-in screen after Sign out', async () => await password.isVisible());
  assert.equal(dialogs, 0, 'Sign out asks nothing');
  assert.equal((await app.call('GET', '/auth/session', { cookie: otherPhone })).res.status, 200,
    'another phone stays signed in');
  await page.reload({ waitUntil: 'load' });
  await waitFor('the sign-in screen after a reopen', async () => await password.isVisible());
  assert.deepEqual(crashes, [], crashes.join(' | '));
});

test('too_many_failures_name_the_maldives_time_to_try_again', async t => {
  const app = await startApp();
  const browser = await chromium.launch();
  t.after(async () => { await browser.close(); await app.close(); });

  // The phone's own clock is somewhere else; the time shown is still Maldivian.
  const context = await browser.newContext({ viewport: PHONE, timezoneId: 'America/New_York' });
  await context.route(url => !String(url).startsWith(app.origin), route =>
    route.fulfill({ status: 200, contentType: 'application/json', body: '{"days":[]}' }));
  const page = await context.newPage();
  await page.goto(app.origin + '/', { waitUntil: 'load' });
  const password = page.getByLabel('Password');
  await waitFor('the sign-in form', async () => await password.isVisible());

  await page.getByLabel('Username').fill(USERNAME);
  const error = page.locator('#signin-error');
  for (let i = 0; i < 5; i++) {
    await password.fill('nope-' + i);
    await page.getByRole('button', { name: 'Sign in' }).click();
    await waitFor(`failure ${i + 1}`, async () =>
      flat(await error.innerText()) === 'Wrong username or password.' && await password.inputValue() === '');
  }
  await password.fill(PASSWORD);
  await page.getByRole('button', { name: 'Sign in' }).click();
  const text = await waitFor('the try-again line', async () => {
    const t = flat(await error.innerText());
    return /^Too many tries\. Try again at \d\d:\d\d\.$/.test(t) ? t : null;
  });
  const row = await app.db.prepare('SELECT "window" FROM signin_attempt').first();
  const end = new Date((Number(row.window) + 1) * 15 * 60 * 1000 + 5 * 3600 * 1000);
  const hhmm = end.toISOString().slice(11, 16);
  assert.equal(text, `Too many tries. Try again at ${hhmm}.`,
    'the time is the end of the window on the Maldives clock (UTC+5)');
});

test('the_sign_in_screen_follows_dark_mode', async t => {
  const app = await startApp();
  const browser = await chromium.launch();
  t.after(async () => { await browser.close(); await app.close(); });
  const light = await freshPhone(browser, app, 'light');
  const dark = await freshPhone(browser, app, 'dark');
  const read = p => p.evaluate(() => ({
    bg: getComputedStyle(document.getElementById('signin')).backgroundColor,
    card: getComputedStyle(document.querySelector('.signin-card')).backgroundColor,
    radius: getComputedStyle(document.querySelector('.signin-card')).borderTopLeftRadius
  }));
  await waitFor('both forms', async () =>
    await light.page.getByLabel('Password').isVisible() && await dark.page.getByLabel('Password').isVisible());
  const l = await read(light.page);
  const d = await read(dark.page);
  assert.notEqual(l.bg, d.bg, 'the background follows the scheme');
  assert.notEqual(l.card, d.card, 'and so does the card');
  assert.equal(l.radius, '18px', 'the card has the house 18px corners');
});
