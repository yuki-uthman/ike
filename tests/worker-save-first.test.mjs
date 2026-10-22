// Oracle for "V2 Save-first service".
//
// Authority: docs/product/brief.md#Decisions (D7, D8, D12, D16, D27) and Scope ->
// DESIGN "V2 Save-first service".
//
// Driving port: HTTP. The app is served by a real workerd runtime (miniflare)
// on a real loopback socket, started by tests/support/app.mjs exactly as
// Cloudflare runs it, and spoken to with plain `fetch` by a caller that is
// signed in (D27). Nothing under worker/src is imported here. The only other
// thing this oracle touches is the driven port the design declares — the D1
// binding — to read back what the store actually holds.
//
// Every name asserted below is declared by the handover's HTTP contract: the
// paths /expenses/entries, /expenses/categories (D17), the request members
// category/amount/receipt/client_entry_id, the response members, the error
// codes and D12's status word "Waiting to send".
//
// Narrowed by D27 (2026-10-22): the PIN, its wrong-PIN budget, the CORS headers,
// the preflight and ALLOWED_ORIGIN are gone. The sign-in that replaces them,
// the Origin gate and the session gate are tests/worker-signin.test.mjs's.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';

import { startApp } from './support/app.mjs';

// D7: the 21 Odoo expense categories, in D7's order.
const CATEGORIES_D7 = [
  'Advertising & Marketing', 'Communication', 'Electricity', 'Fuel / Petrol',
  'Gate Pass (Boat Delivery)', 'Gifts', 'Internet', 'Meals',
  'Medical Checkup (Visa)', 'Salary', 'Shop Maintenance & Repairs', 'Shop Rent',
  'Software Subscriptions', 'Staff Accommodation Rent',
  'Stationery & Packing Supplies', 'Travel & Accommodation',
  'Vehicle Maintenance', 'Visa & Work Permit', 'Warehouse Rent',
  'Waste Disposal', 'Water'
];

// D12: every V2 entry starts here, and V2 produces no other status.
const STATUS_WAITING = 'Waiting to send';

// The members an entry has — exactly these seven, no more. The receipt bytes
// never leave D1 in V2, so only their presence and length appear here.
// next_retry_at belongs to the sending vocabulary; V2 sends nothing, so it is
// null on every V2 entry, but the member is part of the one entry shape.
const ENTRY_MEMBERS = [
  'amount_mvr', 'category', 'client_entry_id', 'next_retry_at',
  'receipt_bytes', 'receipt_present', 'status'
];

const RECEIPT_CAP = 1000000; // decision 16: the Worker's own declared cap

/** The Maldives UTC+5 date (D16) at this instant, as YYYY-MM-DD. */
function maldivesDate(at = Date.now()) {
  return new Date(at + 5 * 3600 * 1000).toISOString().slice(0, 10);
}

function b64(bytes) {
  return Buffer.from(bytes).toString('base64');
}

/** The app, plus a caller already signed in (D27). */
async function startSignedIn(opts) {
  const app = await startApp(opts);
  const cookie = await app.signIn();
  const call = (method, pathname, o = {}) => app.call(method, pathname, { cookie, ...o });
  return { ...app, call };
}

/** Every answer is JSON, and a signed-in answer is never cached (D27). */
function assertJsonAnswer({ res, json }) {
  assert.match(res.headers.get('content-type') || '', /application\/json/,
    'every answer is content-type: application/json');
  assert.equal(res.headers.get('access-control-allow-origin'), null,
    'no answer carries CORS any more: the page shares the Worker\'s origin (D27)');
  assert.equal(res.headers.get('cache-control'), 'no-store',
    'no answer may be kept by a cache between the phone and the Worker');
  assert.ok(json !== null, 'the body parses as JSON');
}

function assertRefusal(answer, status, code) {
  assertJsonAnswer(answer);
  assert.equal(answer.res.status, status, `${code} is answered ${status}`);
  assert.equal(answer.json.saved, false, `${code} cannot answer 'saved'`);
  assert.equal(answer.json.error, code, `the refusal names itself '${code}'`);
}

function assertEntryShape(entry, expected) {
  assert.deepEqual(Object.keys(entry).sort(), ENTRY_MEMBERS,
    'an entry carries exactly the declared members and never the receipt bytes');
  assert.equal(entry.status, STATUS_WAITING, `every V2 entry is '${STATUS_WAITING}' (D12)`);
  assert.equal(entry.next_retry_at, null, 'V2 sends nothing, so next_retry_at is null');
  for (const [k, v] of Object.entries(expected)) {
    assert.deepEqual(entry[k], v, `entry.${k}`);
  }
}

test('worker_answers_saved_only_after_durable_store', async t => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'ike-v2-'));
  const mainDir = path.join(root, 'main');
  const brokenDir = path.join(root, 'broken');

  const running = [];
  t.after(async () => {
    for (const mf of running) { try { await mf.dispose(); } catch { /* done */ } }
    await fs.rm(root, { recursive: true, force: true });
  });
  const start = async opts => {
    const w = await startSignedIn(opts);
    running.push(w.mf);
    return w;
  };

  // ============================================================= the live store
  let w = await start({ persist: mainDir });

  // ---- a save answers 'saved' with the stored entry, receipt and all (D3) ----
  const receipt = Buffer.alloc(100, 0xab);
  const save = {
    category: 'Salary', amount: '250.50',
    receipt: b64(receipt), client_entry_id: 'ce-1'
  };
  const first = await w.call('POST', '/expenses/entries', { body: save });
  assertJsonAnswer(first);
  assert.equal(first.res.status, 200);
  assert.equal(first.json.saved, true, 'the save is the one place that answers saved: true');
  assertEntryShape(first.json.entry, {
    client_entry_id: 'ce-1', amount_mvr: '250.50', category: 'Salary',
    receipt_present: true, receipt_bytes: 100
  });

  // ...and 'saved' is true only because the bytes are already in the store.
  const stored = await w.db
    .prepare('SELECT amount_laari, typeof(amount_laari) AS t, length(receipt) AS n, entry_date'
      + ' FROM entry WHERE client_entry_id = ?').bind('ce-1').first();
  assert.equal(stored.amount_laari, 25050, 'MVR 250.50 is stored as integer laari (decision 6)');
  assert.equal(stored.t, 'integer', 'laari is an INTEGER, not a REAL');
  assert.equal(stored.n, 100, 'the receipt bytes are in the same durable row as the entry');
  assert.equal(stored.entry_date, maldivesDate(), 'the day is the UTC+5 date of saving (D16)');

  // ---- the same client entry id twice stores one entry, not two (D10) ----
  const second = await w.call('POST', '/expenses/entries', { body: save });
  assert.equal(second.res.status, 200);
  assert.deepEqual(second.json, first.json,
    'the second send answers about the row the first send stored');
  const rows = await w.db.prepare('SELECT COUNT(*) AS n FROM entry').first();
  assert.equal(rows.n, 1, 'one row, not two');

  // ---- the day's entries, in saved_at then client_entry_id order ----
  const later = await w.call('POST', '/expenses/entries', {
    body: { category: 'Meals', amount: 42, client_entry_id: 'ce-2' }
  });
  assert.equal(later.res.status, 200);
  assertEntryShape(later.json.entry, {
    client_entry_id: 'ce-2', amount_mvr: '42.00', category: 'Meals',
    receipt_present: false, receipt_bytes: 0
  });

  const today = maldivesDate();
  const day = await w.call('GET', `/expenses/entries?date=${today}`);
  assertJsonAnswer(day);
  assert.equal(day.res.status, 200);
  assert.equal(day.json.date, today);
  assert.deepEqual(day.json.entries.map(e => e.client_entry_id), ['ce-1', 'ce-2'],
    "the day's entries come back ordered by saved_at then client_entry_id");
  assert.deepEqual(day.json.entries[0], first.json.entry,
    'the listed entry is byte-identical to the one the save answered, so it came out of the store');
  for (const e of day.json.entries) assertEntryShape(e, {});

  const implied = await w.call('GET', '/expenses/entries');
  assert.equal(implied.res.status, 200);
  assert.equal(implied.json.date, today, "an omitted date resolves to D16's UTC+5 day and is echoed");
  assert.deepEqual(implied.json.entries, day.json.entries);

  // ---- the 21 categories, in D7's order (D7, decision 13) ----
  const cats = await w.call('GET', '/expenses/categories');
  assertJsonAnswer(cats);
  assert.equal(cats.res.status, 200);
  assert.deepEqual(cats.json.categories, CATEGORIES_D7,
    'the Worker serves exactly D7\'s 21 names in D7\'s order');

  // ---- the boundaries, none of which can answer 'saved' (decision 16) ----
  const body = over => ({ ...save, client_entry_id: `ce-x-${Math.random()}`, ...over });

  // A forged Origin is refused before anything else is judged (D27).
  for (const origin of ['https://evil.example', null]) {
    const bad = await w.call('POST', '/expenses/entries',
      { origin, body: body({ category: 'nope', amount: '-1' }) });
    assertRefusal(bad, 403, 'origin_not_allowed');
  }

  assertRefusal(await w.call('POST', '/expenses/entries', { body: body({ category: 'Mileage' }) }),
    422, 'unknown_category');

  for (const amount of [0, -5, 12.345, 'abc', '0.00', '']) {
    assertRefusal(await w.call('POST', '/expenses/entries', { body: body({ amount }) }),
      422, 'amount_not_positive_mvr');
  }

  for (const client_entry_id of [undefined, '', '   ']) {
    assertRefusal(await w.call('POST', '/expenses/entries', { body: { ...save, client_entry_id } }),
      422, 'invalid_client_entry_id');
  }
  assertRefusal(await w.call('POST', '/expenses/entries', { raw: '{not json' }), 422, 'bad_json');
  assertRefusal(await w.call('POST', '/expenses/entries', { body: body({ receipt: '!!!not base64!!!' }) }),
    422, 'bad_receipt_base64');

  assertRefusal(await w.call('GET', '/nowhere'), 404, 'not_found');
  assertRefusal(await w.call('DELETE', '/expenses/entries'), 404, 'not_found');

  // ---- the receipt cap is the Worker's own, and it is exact ----
  const atCap = await w.call('POST', '/expenses/entries', {
    body: { category: 'Fuel / Petrol', amount: '1999.99',
      receipt: b64(Buffer.alloc(RECEIPT_CAP, 0x7f)), client_entry_id: 'ce-cap' }
  });
  assert.equal(atCap.res.status, 200, `${RECEIPT_CAP} decoded bytes still saves`);
  assert.equal(atCap.json.entry.receipt_bytes, RECEIPT_CAP);
  assert.equal(atCap.json.entry.amount_mvr, '1999.99');
  assert.equal(
    (await w.db.prepare('SELECT length(receipt) AS n FROM entry WHERE client_entry_id = ?')
      .bind('ce-cap').first()).n,
    RECEIPT_CAP, 'the capped receipt is stored whole');

  const overCap = await w.call('POST', '/expenses/entries', {
    body: { category: 'Fuel / Petrol', amount: '10.00',
      receipt: b64(Buffer.alloc(RECEIPT_CAP + 1, 0x7f)), client_entry_id: 'ce-over' }
  });
  assertRefusal(overCap, 413, 'receipt_too_large');
  assert.equal(overCap.json.receipt_bytes, RECEIPT_CAP + 1);
  assert.equal(overCap.json.limit_bytes, RECEIPT_CAP);
  assert.equal(
    (await w.db.prepare('SELECT COUNT(*) AS n FROM entry WHERE client_entry_id = ?')
      .bind('ce-over').first()).n,
    0, 'the refused receipt stored no row');

  // ================================================== durability across runtimes
  const beforeStop = first.json.entry;
  await w.mf.dispose();

  w = await start({ persist: mainDir, schema: false });
  const afterRestart = await w.call('GET', `/expenses/entries?date=${today}`);
  assert.equal(afterRestart.res.status, 200);
  assert.deepEqual(
    afterRestart.json.entries.find(e => e.client_entry_id === 'ce-1'), beforeStop,
    "a brand-new runtime over the same store still answers the entry it said was saved");
  assert.equal(
    (await w.db.prepare('SELECT length(receipt) AS n FROM entry WHERE client_entry_id = ?')
      .bind('ce-1').first()).n,
    100, 'the receipt survived the runtime, not just the entry');

  // ============================== the store failing is Indeterminate, not 'saved'
  const broken = await start({ persist: brokenDir });
  await broken.db.prepare('DROP TABLE entry').run();

  const notStored = await broken.call('POST', '/expenses/entries', {
    body: { category: 'Gifts', amount: '77.00', client_entry_id: 'ce-fail' }
  });
  assertRefusal(notStored, 503, 'not_stored');
  assert.ok(!('entry' in notStored.json),
    'a save that was not stored answers no entry');

  assertRefusal(await broken.call('GET', `/expenses/entries?date=${today}`), 503, 'not_read');

});
