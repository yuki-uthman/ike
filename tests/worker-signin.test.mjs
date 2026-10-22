// Oracle for "one shared sign-in" (issue #7).
//
// Authority: docs/product/brief.md#Decisions D22-D28 (the owner's decisions of
// 2026-10-21, recorded in yuki-uthman/ike#7).
//
// Driving port: HTTP, against a real workerd (miniflare) that serves the page
// AND the Worker on one loopback origin, exactly as Cloudflare does
// (tests/support/app.mjs). The driven ports touched directly are the D1 binding
// — to stage a session's age, which no test can wait 30 days for, and to read
// what the store keeps — and the owner's own script, worker/scripts/user.mjs.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import path from 'node:path';
import { promisify } from 'node:util';

import { PASSWORD, REPO, USERNAME, addUser, sessionCookieOf, startApp } from './support/app.mjs';
import { loadWords } from '../worker/scripts/user.mjs';

const run = promisify(execFile);
const USER_SCRIPT = path.join(REPO, 'worker', 'scripts', 'user.mjs');

const DAY_MS = 24 * 3600 * 1000;

/** Every data route the Worker serves today (D17), and the gated quotations area. */
const DATA_ROUTES = [
  ['GET', '/expenses/categories'],
  ['GET', '/expenses/entries'],
  ['GET', '/expenses/entries?date=2026-10-22'],
  ['POST', '/expenses/entries'],
  ['POST', '/expenses/entries/anything/retry'],
  ['GET', '/quotations/'],
  ['GET', '/quotations/open']
];

function setCookieOf(res) {
  return typeof res.headers.getSetCookie === 'function'
    ? res.headers.getSetCookie()
    : [res.headers.get('set-cookie')].filter(Boolean);
}

test('signin_admits_the_right_password_and_refuses_everything_else_alike', async t => {
  const app = await startApp();
  t.after(() => app.close());

  // ---- right username, right password: a session cookie and the name ----
  const ok = await app.call('POST', '/auth/signin',
    { body: { username: USERNAME, password: PASSWORD } });
  assert.equal(ok.res.status, 200);
  assert.deepEqual(ok.json, { signed_in: true, username: USERNAME });
  const [cookie] = setCookieOf(ok.res);
  assert.ok(cookie, 'a successful sign-in hands the phone a cookie');
  const parts = cookie.split(';').map(p => p.trim().toLowerCase());
  assert.ok(parts[0].startsWith('ike_session='), 'the cookie is the session cookie');
  for (const flag of ['httponly', 'secure', 'samesite=strict', 'path=/']) {
    assert.ok(parts.includes(flag), `the session cookie is ${flag} (D24)`);
  }
  assert.ok(parts.includes(`max-age=${30 * 24 * 3600}`), 'it lives 30 days (D24)');
  const token = parts[0].slice('ike_session='.length);
  assert.ok(token.length >= 43, 'the token carries 32 random bytes');

  // ...and only the token's hash is stored, never the token, never the password.
  const sessions = await app.db.prepare('SELECT token_hash FROM session').all();
  assert.equal(sessions.results.length, 1, 'one sign-in, one stored session');
  assert.notEqual(sessions.results[0].token_hash, token, 'the token itself is not stored');
  const users = await app.db.prepare('SELECT * FROM "user"').all();
  assert.ok(!JSON.stringify(users.results).includes(PASSWORD), 'the password is not stored');

  const session = await app.call('GET', '/auth/session', { cookie: cookie.split(';')[0] });
  assert.equal(session.res.status, 200);
  assert.deepEqual(session.json, { signed_in: true, username: USERNAME });

  // ---- a wrong password and an unknown name are ONE answer (D25) ----
  const wrong = await app.call('POST', '/auth/signin',
    { body: { username: USERNAME, password: 'not-the-password' }, ip: '10.0.0.1' });
  const unknown = await app.call('POST', '/auth/signin',
    { body: { username: 'nobody', password: PASSWORD }, ip: '10.0.0.2' });
  for (const r of [wrong, unknown]) {
    assert.equal(r.res.status, 401);
    assert.equal(r.json.error, 'wrong_username_or_password');
    assert.deepEqual(setCookieOf(r.res), [], 'a refused sign-in sets no cookie');
  }
  assert.deepEqual(wrong.json, unknown.json,
    'an unknown name and a wrong password cannot be told apart');

  // ---- a disabled account signs nobody in ----
  await app.db.prepare('UPDATE "user" SET disabled_at = ? WHERE username = ?')
    .bind(new Date().toISOString(), USERNAME).run();
  const disabled = await app.call('POST', '/auth/signin',
    { body: { username: USERNAME, password: PASSWORD }, ip: '10.0.0.3' });
  assert.equal(disabled.res.status, 401);
  assert.deepEqual(disabled.json, wrong.json);
  assert.equal((await app.call('GET', '/auth/session', { cookie: cookie.split(';')[0] })).res.status,
    401, 'a disabled account\'s sessions stop working at once');

  // ---- bad JSON is not a failed sign-in ----
  const bad = await app.call('POST', '/auth/signin', { raw: '{nope', ip: '10.0.0.4' });
  assert.equal(bad.res.status, 422);
  assert.equal(bad.json.error, 'bad_json');
});

test('with_no_account_every_signin_fails_with_the_normal_message', async t => {
  const app = await startApp({ user: false });
  t.after(() => app.close());
  const r = await app.call('POST', '/auth/signin', { body: { username: USERNAME, password: PASSWORD } });
  assert.equal(r.res.status, 401);
  assert.equal(r.json.error, 'wrong_username_or_password');
});

test('five_failures_per_source_per_window_then_429_even_for_the_right_password', async t => {
  const app = await startApp();
  t.after(() => app.close());
  const ip = '203.0.113.9';

  for (let i = 1; i <= 5; i++) {
    const r = await app.call('POST', '/auth/signin',
      { body: { username: i % 2 ? USERNAME : 'nobody', password: 'wrong' }, ip });
    assert.equal(r.res.status, 401, `failure ${i} is still the plain refusal`);
  }
  const limited = await app.call('POST', '/auth/signin',
    { body: { username: USERNAME, password: PASSWORD }, ip });
  assert.equal(limited.res.status, 429, 'the budget is read before the password (D25)');
  assert.equal(limited.json.error, 'too_many_attempts');
  assert.deepEqual(setCookieOf(limited.res), [], 'a limited sign-in sets no cookie');
  assert.match(limited.res.headers.get('retry-after') || '', /^\d+$/, 'Retry-After is whole seconds');
  const retryAt = Date.parse(limited.json.retry_at);
  const windowMs = 15 * 60 * 1000;
  assert.ok(Number.isFinite(retryAt), 'retry_at is an ISO instant');
  assert.equal(retryAt % windowMs, 0, 'retry_at is the end of a fixed 15-minute window');
  assert.ok(retryAt > Date.now() && retryAt <= Date.now() + windowMs + 1000,
    'retry_at is the end of the CURRENT window');

  // Another source is not affected, and a success consumes no budget.
  const other = await app.call('POST', '/auth/signin',
    { body: { username: USERNAME, password: PASSWORD }, ip: '203.0.113.10' });
  assert.equal(other.res.status, 200, 'the limit is per source');
  const row = await app.db.prepare('SELECT misses FROM signin_attempt WHERE source = ?')
    .bind('203.0.113.10').first();
  assert.ok(!row || Number(row.misses) === 0, 'a successful sign-in consumes no budget');

  // The budget comes back with the next window.
  await app.db.prepare('UPDATE signin_attempt SET "window" = "window" - 1 WHERE source = ?')
    .bind(ip).run();
  const next = await app.call('POST', '/auth/signin',
    { body: { username: USERNAME, password: PASSWORD }, ip });
  assert.equal(next.res.status, 200, 'the next window admits the right password again');
});

test('every_data_route_refuses_without_a_session_and_the_shell_is_served_without_one', async t => {
  const app = await startApp();
  t.after(() => app.close());

  for (const [method, route] of DATA_ROUTES) {
    const r = await app.call(method, route, method === 'POST' ? { body: {} } : {});
    assert.equal(r.res.status, 401, `${method} ${route} needs a session`);
    assert.equal(r.json.error, 'signed_out');
  }
  const stale = await app.call('GET', '/expenses/categories', { cookie: 'ike_session=forged' });
  assert.equal(stale.res.status, 401, 'an unknown token is no session');

  // The page itself, and everything it needs to open, needs no session.
  for (const file of ['/', '/index.html', '/sw.js', '/manifest.webmanifest', '/icons/icon-192.png']) {
    const r = await fetch(app.origin + file);
    assert.equal(r.status, 200, `${file} is served without a session`);
  }
  const shell = await (await fetch(app.origin + '/')).text();
  assert.match(shell, /id="signin"/, 'the shell carries the sign-in screen');
  assert.ok(!/ike-expenses-service/.test(shell),
    'the page names no service address: it calls its own origin (D27)');
  assert.ok(!/e-pin|Your PIN/.test(shell), 'the PIN field is gone (D27)');

  // A path nobody serves is the plain 404, signed in or not.
  const cookie = await app.signIn();
  assert.equal((await app.call('GET', '/nowhere', { cookie })).res.status, 404);
  assert.equal((await app.call('GET', '/auth/nowhere')).res.status, 404);
  // The quotations area is gated but has no route yet: past the gate it is 404.
  assert.equal((await app.call('GET', '/quotations/open', { cookie })).res.status, 404);
});

test('a_non_get_without_the_workers_own_origin_is_refused_before_anything_else', async t => {
  const app = await startApp();
  t.after(() => app.close());
  const cookie = await app.signIn();
  const save = { category: 'Water', amount: '5', client_entry_id: 'forged-1' };

  for (const origin of ['https://evil.example', null, app.origin.replace('127.0.0.1', 'localhost')]) {
    for (const [route, body] of [['/expenses/entries', save], ['/auth/signin',
      { username: USERNAME, password: PASSWORD }], ['/auth/signout', {}]]) {
      const r = await app.call('POST', route, { origin, body, cookie });
      assert.equal(r.res.status, 403, `POST ${route} from Origin ${origin} is refused`);
      assert.equal(r.json.error, 'origin_not_allowed');
      assert.equal(r.res.headers.get('access-control-allow-origin'), null, 'and carries no CORS');
    }
  }
  assert.equal((await app.db.prepare('SELECT COUNT(*) AS n FROM entry').first()).n, 0,
    'a forged-origin save stored nothing');
  assert.equal((await app.call('GET', '/auth/session', { cookie })).res.status, 200,
    'a forged sign-out ended nothing');

  // OPTIONS is a non-GET like any other: there is no preflight to answer.
  const pre = await app.call('OPTIONS', '/expenses/entries', { origin: 'https://evil.example' });
  assert.equal(pre.res.status, 403);
});

test('signout_ends_this_phone_and_the_script_signout_ends_every_phone', async t => {
  const app = await startApp();
  t.after(() => app.close());
  const phoneA = await app.signIn();
  const phoneB = await app.signIn();
  const phoneC = await app.signIn();

  const out = await app.call('POST', '/auth/signout', { cookie: phoneA });
  assert.equal(out.res.status, 200);
  const cleared = setCookieOf(out.res)[0] || '';
  assert.match(cleared, /^ike_session=;/, 'sign-out clears the cookie');
  assert.match(cleared, /Max-Age=0/i);
  assert.equal((await app.call('GET', '/auth/session', { cookie: phoneA })).res.status, 401,
    'the phone that signed out is out');
  assert.equal((await app.call('GET', '/auth/session', { cookie: phoneB })).res.status, 200,
    'the other phones are not (D28: Sign out signs out that phone only)');

  // The owner's `signout` ends every session of the account at once (D26).
  const printed = await run(process.execPath, [USER_SCRIPT, 'signout', USERNAME]);
  const sql = printed.stdout.split('\n').find(l => l.startsWith('DELETE'));
  assert.ok(sql, 'signout prints one SQL statement to run');
  await app.db.exec(sql);
  for (const phone of [phoneB, phoneC]) {
    assert.equal((await app.call('GET', '/auth/session', { cookie: phone })).res.status, 401);
  }
  assert.equal((await app.call('POST', '/auth/signin',
    { body: { username: USERNAME, password: PASSWORD } })).res.status, 200,
  'signout ends sessions, not the account');
});

test('a_session_expires_30_days_after_last_use_and_is_renewed_at_most_daily', async t => {
  const app = await startApp();
  t.after(() => app.close());
  const cookie = await app.signIn();
  const age = async ms => {
    const at = new Date(Date.now() - ms).toISOString();
    await app.db.prepare('UPDATE session SET last_used_at = ?, created_at = ?').bind(at, at).run();
  };
  const row = () => app.db.prepare('SELECT last_used_at, expires_at FROM session').first();

  // Used within the day: no write, no new cookie.
  const before = await row();
  const fresh = await app.call('GET', '/expenses/categories', { cookie });
  assert.equal(fresh.res.status, 200);
  assert.deepEqual(await row(), before, 'a session used today is not rewritten');
  assert.deepEqual(setCookieOf(fresh.res), [], 'and the phone is not sent its cookie again');

  // Last used over a day ago: renewed, and the phone gets the new lifetime.
  await age(DAY_MS + 60 * 1000);
  const renewed = await app.call('GET', '/expenses/categories', { cookie });
  assert.equal(renewed.res.status, 200);
  const after = await row();
  assert.ok(Date.now() - Date.parse(after.last_used_at) < 60 * 1000, 'last use is now');
  assert.ok(Math.abs(Date.parse(after.expires_at) - (Date.now() + 30 * DAY_MS)) < 60 * 1000,
    'the expiry moved to 30 days from now');
  assert.equal(sessionCookieOf(renewed.res), cookie,
    'the renewed answer re-sends the same session cookie with a fresh Max-Age');

  // The page's FIRST call on every open is /auth/session, so that is where a
  // daily phone is usually renewed: it must hand back the cookie too, or the
  // cookie would lapse 30 days after sign-in however often the phone was used.
  await age(DAY_MS + 60 * 1000);
  const viaSession = await app.call('GET', '/auth/session', { cookie });
  assert.equal(viaSession.res.status, 200);
  assert.equal(sessionCookieOf(viaSession.res), cookie,
    'a renewal through /auth/session re-sends the session cookie');

  // Past its expiry: no session, on every route.
  await app.db.prepare('UPDATE session SET expires_at = ?')
    .bind(new Date(Date.now() - 1000).toISOString()).run();
  assert.equal((await app.call('GET', '/auth/session', { cookie })).res.status, 401);
  assert.equal((await app.call('GET', '/expenses/categories', { cookie })).res.status, 401);
});

test('the_owners_script_prints_sql_with_a_generated_password_that_signs_in', async t => {
  const app = await startApp({ user: false });
  t.after(() => app.close());

  const words = loadWords();
  assert.equal(words.length, 7776, 'the list beside the script is the 7,776-word list');

  const first = await run(process.execPath, [USER_SCRIPT, 'add', 'manager']);
  const lines = first.stdout.split('\n');
  const password = lines[lines.findIndex(l => l.startsWith('Password for manager')) + 1].trim();
  const parts = password.split('-');
  assert.equal(parts.length, 5, 'a password is five words joined by "-" (D23)');
  for (const w of parts) assert.ok(words.includes(w), `"${w}" is from the list`);
  const sql = lines.find(l => l.startsWith('INSERT INTO user '));
  assert.ok(sql, 'add prints the SQL to run');
  assert.ok(!sql.includes(password), 'the printed SQL holds the hash, never the password');
  assert.match(sql, /, 10000,/, 'new passwords are hashed with 10,000 rounds (D23)');
  await app.db.exec(sql);

  const cookie = await app.signIn('manager', password);
  assert.ok(cookie, 'the generated password signs in');

  // `add` again replaces the password: the old one stops, the new one works.
  const second = await run(process.execPath, [USER_SCRIPT, 'add', 'manager']);
  const l2 = second.stdout.split('\n');
  const newPassword = l2[l2.findIndex(l => l.startsWith('Password for manager')) + 1].trim();
  assert.notEqual(newPassword, password, 'every add draws a fresh password');
  await app.db.exec(l2.find(l => l.startsWith('INSERT INTO user ')));
  assert.equal((await app.db.prepare('SELECT COUNT(*) AS n FROM "user"').first()).n, 1,
    'the same name is one account');
  assert.equal((await app.call('POST', '/auth/signin',
    { body: { username: 'manager', password }, ip: '10.0.0.8' })).res.status, 401);
  assert.equal((await app.call('POST', '/auth/signin',
    { body: { username: 'manager', password: newPassword } })).res.status, 200);

  // `remove` ends the sessions and the account.
  const removed = await run(process.execPath, [USER_SCRIPT, 'remove', 'manager']);
  await app.db.exec(removed.stdout.split('\n').find(l => l.startsWith('DELETE')));
  assert.equal((await app.call('GET', '/auth/session', { cookie })).res.status, 401);
  assert.equal((await app.db.prepare('SELECT COUNT(*) AS n FROM "user"').first()).n, 0);

  // A name the script cannot quote safely is refused, not escaped.
  await assert.rejects(run(process.execPath, [USER_SCRIPT, 'add', "o'brien"]),
    err => err.code === 2, 'an unsafe username is refused with exit 2');

  // The tests' own account goes through the same statement.
  await addUser(app.db, 'second', 'two-words');
  assert.ok(await app.signIn('second', 'two-words'));
});

test('a_burst_of_parallel_sign_ins_buys_no_more_than_five_tries', async t => {
  const app = await startApp();
  t.after(() => app.close());
  const ip = '198.18.0.1';
  const answers = await Promise.all(Array.from({ length: 20 }, () =>
    app.call('POST', '/auth/signin', { body: { username: USERNAME, password: 'wrong' }, ip })));
  const checked = answers.filter(a => a.res.status === 401).length;
  const limited = answers.filter(a => a.res.status === 429).length;
  assert.ok(checked <= 5, `at most five passwords were checked (${checked})`);
  assert.equal(checked + limited, 20, 'every other try in the burst was refused as limited');
});

test('the_schedule_prunes_expired_sessions_and_spent_sign_in_windows', async t => {
  const app = await startApp();
  t.after(() => app.close());
  const live = await app.signIn();
  await app.signIn();
  await app.db.prepare('UPDATE session SET expires_at = ? WHERE rowid = (SELECT MAX(rowid) FROM session)')
    .bind(new Date(Date.now() - 1000).toISOString()).run();
  await app.db.prepare('INSERT INTO signin_attempt (source, "window", misses) VALUES (?, ?, 3)')
    .bind('198.18.0.2', Math.floor(Date.now() / (15 * 60 * 1000)) - 1).run();
  await app.call('POST', '/auth/signin', { body: { username: USERNAME, password: 'x' }, ip: '198.18.0.3' });

  assert.equal(await app.tick(), 200, 'the scheduled run completes');
  assert.equal((await app.db.prepare('SELECT COUNT(*) AS n FROM session').first()).n, 1,
    'the expired session is gone and the live one is kept');
  assert.equal((await app.call('GET', '/auth/session', { cookie: live })).res.status, 200);
  const windows = (await app.db.prepare('SELECT source FROM signin_attempt').all())
    .results.map(r => r.source);
  assert.ok(!windows.includes('198.18.0.2'), 'a window before the current one is gone');
  assert.ok(windows.includes('198.18.0.3'), 'the current window\'s failure is kept');
});

test('a_store_that_cannot_answer_is_a_json_503_and_a_malformed_retry_is_a_404', async t => {
  const app = await startApp();
  t.after(() => app.close());
  const cookie = await app.signIn();

  const bad = await app.call('POST', '/expenses/entries/%E0%A4/retry', { cookie });
  assert.equal(bad.res.status, 404, 'a malformed escape names no entry');
  assert.deepEqual(bad.json, { saved: false, error: 'not_found' });

  await app.db.prepare('DROP TABLE session').run();
  for (const [method, route, body] of [['GET', '/expenses/categories'], ['GET', '/auth/session'],
    ['POST', '/auth/signin', { username: USERNAME, password: PASSWORD }]]) {
    const r = await app.call(method, route, { cookie, body });
    assert.equal(r.res.status, 503, `${method} ${route} is Indeterminate when the store fails`);
    assert.match(r.res.headers.get('content-type') || '', /application\/json/, 'and still JSON');
    assert.equal(r.json.error, 'not_read');
  }
});
