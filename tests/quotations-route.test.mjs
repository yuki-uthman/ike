// GET /quotations/open and the quotations schedule (issue #8; D31), driven
// through a real workerd (tests/support/app.mjs) whose Odoo is a LOCAL FAKE
// speaking JSON-2 (tests/support/fake-odoo-json2.mjs). No call in this file can
// reach mrh-investment.odoo.com.
//
// Driving ports: HTTP (GET /quotations/open, POST /expenses/entries) and the
// two crons in worker/wrangler.json, fired through workerd's own
// /cdn-cgi/handler/scheduled port. Nothing under worker/src is imported.

import { test } from 'node:test';
import assert from 'node:assert/strict';

import { startApp } from './support/app.mjs';
import { ODOO_DB, ODOO_KEY, quotationRecords, startFakeOdooJson2 } from './support/fake-odoo-json2.mjs';

const EXPENSES_CRON = '*/15 * * * *';
const QUOTATIONS_CRON = '7-59/15 * * * *';

async function start(t) {
  const odoo = await startFakeOdooJson2(quotationRecords());
  const app = await startApp({
    bindings: { ODOO_URL: odoo.url, ODOO_DB, ODOO_USERNAME: 'mrhpvt@gmail.com', ODOO_API_KEY: ODOO_KEY }
  });
  t.after(async () => { await app.close(); await odoo.close(); });
  const fire = async cron => {
    const res = await fetch(`${app.origin}/cdn-cgi/handler/scheduled?cron=${encodeURIComponent(cron)}`);
    await res.text();
    return res.status;
  };
  return { odoo, app, fire, cookie: await app.signIn() };
}

test('the_open_quotations_need_a_session', async t => {
  const { app } = await start(t);
  const r = await app.call('GET', '/quotations/open');
  assert.equal(r.res.status, 401);
  assert.equal(r.json.error, 'signed_out');
});

test('before_the_first_build_the_answer_is_503_not_ready', async t => {
  const { app, cookie } = await start(t);
  const r = await app.call('GET', '/quotations/open', { cookie });
  assert.equal(r.res.status, 503);
  assert.deepEqual(r.json, { error: 'not_ready' });
  assert.equal(r.res.headers.get('cache-control'), 'no-store');
});

test('the_quotations_schedule_builds_the_feed_and_the_route_serves_it', async t => {
  const { app, cookie, fire } = await start(t);
  await fire(QUOTATIONS_CRON);

  const r = await app.call('GET', '/quotations/open', { cookie });
  assert.equal(r.res.status, 200);
  assert.match(r.res.headers.get('content-type'), /application\/json/);
  assert.equal(r.res.headers.get('cache-control'), 'no-store', 'a signed-in answer is never cached');
  const feed = r.json;
  assert.deepEqual(feed.quotations.map(q => q.ref), ['S00101', 'S00104', 'S00103']);
  assert.ok(feed.quotations.every(q => q.kind === 'quotation'));
  assert.equal(feed.openCount, 3);
  assert.equal(feed.quotationCount, 3);
  assert.equal(feed.openTotal, 1350.5);
  assert.equal(feed.quotationTotal, 1350.5);
  assert.equal(feed.orderCount, undefined, 'quotations only: no order fields');
  assert.match(feed.generatedAt, /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$/);
  assert.equal(feed.days[0].date, '2026-10-19', 'the timeline starts at the oldest open day');

  // The stored row is the snapshot that was served.
  const row = await app.db.prepare('SELECT name, json, generated_at FROM feed').first();
  assert.equal(row.name, 'quotations');
  assert.equal(row.generated_at, feed.generatedAt);
  assert.deepEqual(JSON.parse(row.json), feed);
});

test('each_schedule_runs_only_its_own_work', async t => {
  const { app, cookie, fire, odoo } = await start(t);
  // An expense waiting to be sent: the expenses run would send it to Odoo.
  const saved = await app.call('POST', '/expenses/entries', {
    cookie, body: { category: 'Water', amount: '5', client_entry_id: 'q-cron-1' }
  });
  assert.equal(saved.res.status, 200);

  await fire(QUOTATIONS_CRON);
  const paths = odoo.calls().map(c => c.path);
  assert.ok(paths.length > 0);
  assert.ok(paths.every(p => p.startsWith('/json/2/')),
    'the quotations run reads JSON-2 and never touches the XML-RPC sync: ' + paths.join(' '));
  const entry = await app.db.prepare('SELECT attempts, odoo_id FROM entry WHERE client_entry_id = ?')
    .bind('q-cron-1').first();
  assert.equal(entry.attempts, 0, 'the expense was not sent by the quotations run');
  assert.equal(entry.odoo_id, null);

  odoo.state.calls.length = 0;
  await app.db.prepare('DELETE FROM feed').run();
  await fire(EXPENSES_CRON);
  assert.ok(!odoo.calls().some(c => c.path.startsWith('/json/2/')),
    'the expenses run builds no quotations');
  assert.equal((await app.call('GET', '/quotations/open', { cookie })).res.status, 503);
});

test('a_failed_build_keeps_the_previous_snapshot', async t => {
  const { app, cookie, fire, odoo } = await start(t);
  assert.equal(await fire(QUOTATIONS_CRON), 200, 'a good build is a good run');
  const before = (await app.call('GET', '/quotations/open', { cookie })).text;

  odoo.state.records['sale.order'][3].amount_total = 999;   // a change a failed run must not show
  odoo.fail('account.payment');
  assert.notEqual(await fire(QUOTATIONS_CRON), 200, 'a failed build is a failed run, never a silent one');
  assert.equal((await app.call('GET', '/quotations/open', { cookie })).text, before);

  odoo.heal();
  odoo.fail('sale.order.line');   // the very last read
  await fire(QUOTATIONS_CRON);
  assert.equal((await app.call('GET', '/quotations/open', { cookie })).text, before);

  odoo.heal();
  await fire(QUOTATIONS_CRON);
  const after = (await app.call('GET', '/quotations/open', { cookie })).json;
  assert.equal(after.quotations.find(q => q.ref === 'S00104').total, 999,
    'the next good run replaces the snapshot');
});

test('the_quotations_area_serves_only_its_one_read', async t => {
  const { app, cookie } = await start(t);
  assert.equal((await app.call('GET', '/quotations', { cookie })).res.status, 404);
  assert.equal((await app.call('GET', '/quotations/open/x', { cookie })).res.status, 404);
  assert.equal((await app.call('POST', '/quotations/open', { cookie, body: {} })).res.status, 404);
});
