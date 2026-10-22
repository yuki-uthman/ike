// The open-quotations build (issue #8; D29, D30): the rules of ike-data's
// fetch_quotations.py, ported to the Worker for quotations only, read over
// JSON-2 from a LOCAL FAKE Odoo (tests/support/fake-odoo-json2.mjs).
//
// Unlike the black-box oracles, this suite imports worker/src/quotations.mjs
// and worker/src/odoo-json2.mjs directly: the rounding and summing rules are
// pure functions whose whole point is to equal Python's, and they are pinned
// here value by value.

import { test } from 'node:test';
import assert from 'node:assert/strict';

import { buildOpenQuotations, pyRound2, pySum, splitOrigins } from '../worker/src/quotations.mjs';
import { makeJson2Reader } from '../worker/src/odoo-json2.mjs';
import { ODOO_DB, ODOO_KEY, quotationRecords, startFakeOdooJson2 } from './support/fake-odoo-json2.mjs';

const NOW = Date.parse('2026-10-22T10:00:00Z');   // 15:00 on the 22nd in the Maldives

async function fakeAndReader(t, records = quotationRecords()) {
  const odoo = await startFakeOdooJson2(records);
  t.after(() => odoo.close());
  const reader = makeJson2Reader({ ODOO_URL: odoo.url, ODOO_DB, ODOO_API_KEY: ODOO_KEY });
  return { odoo, reader };
}

test('round_equals_pythons_round_to_two_places_including_exact_binary_ties', () => {
  // Exact binary ties go half to even, as Python's round(x, 2) does.
  assert.equal(pyRound2(0.125), 0.12);
  assert.equal(pyRound2(0.625), 0.62);
  assert.equal(pyRound2(0.375), 0.38);
  assert.equal(pyRound2(0.875), 0.88);
  assert.equal(pyRound2(-0.125), -0.12);
  // 2.675 is not a tie in binary (it is 2.67499...): Python gives 2.67, and so
  // must this, where Math.round(x * 100) / 100 would give 2.68.
  assert.equal(pyRound2(2.675), 2.67);
  assert.equal(pyRound2(1.005), 1);
  assert.equal(pyRound2(2158753.2), 2158753.2);
  assert.equal(pyRound2(0.1 + 0.2), 0.3);
  assert.equal(pyRound2(1000 - 399.999), 600);
});

test('sum_equals_python_3_12s_compensated_float_sum', () => {
  // Python 3.12: sum([0.1] * 10) == 1.0, where a plain loop gives 0.9999999999999999.
  assert.equal(pySum(Array(10).fill(0.1)), 1);
  assert.equal(pySum([1e16, 1, -1e16]), 1);
  assert.equal(pySum([]), 0);
});

test('invoice_origin_splits_on_commas_and_whitespace', () => {
  assert.deepEqual(splitOrigins('S00102, S00999'), ['S00102', 'S00999']);
  assert.deepEqual(splitOrigins('S00102,S00103\nS00104'), ['S00102', 'S00103', 'S00104']);
  assert.deepEqual(splitOrigins(false), []);
  assert.deepEqual(splitOrigins(''), []);
});

test('the_feed_lists_open_quotations_only_with_every_settlement_channel_counted', async t => {
  const { reader } = await fakeAndReader(t);
  const built = await buildOpenQuotations(reader, NOW);
  assert.equal(built.ok, true, JSON.stringify(built));

  assert.deepEqual(built.value, {
    company: 'MRH Investment',
    currency: 'MVR',
    generatedAt: '2026-10-22T10:00:00Z',
    openCount: 3,
    openTotal: 1350.5,
    quotationCount: 3,
    quotationTotal: 1350.5,
    // From the oldest open day to today (Maldives), gaps filled.
    days: [
      { date: '2026-10-19', count: 1, outstanding: 500, total: 800 },
      { date: '2026-10-20', count: 0, outstanding: 0, total: 0 },
      { date: '2026-10-21', count: 2, outstanding: 850.5, total: 1250.5 },
      { date: '2026-10-22', count: 0, outstanding: 0, total: 0 }
    ],
    // Newest day first, biggest first within a day.
    quotations: [
      {
        id: 101, ref: 'S00101', kind: 'quotation', customer: 'Ali Stores', salesperson: 'Mariyam',
        // Raised 20:30 UTC on the 20th: the 21st in the Maldives.
        date: '2026-10-21', expires: '2026-10-25',
        // 400 at the till; the draft POS order's 300 is not money.
        total: 1000, settled: 400, outstanding: 600,
        // The section line is not a product.
        lines: [
          { name: '[PP9] PAPER PLATE 9"', qty: 10, total: 400 },
          { name: 'Cups', qty: 5, total: 600 }
        ]
      },
      {
        id: 104, ref: 'S00104', kind: 'quotation', customer: null, salesperson: 'Mariyam',
        date: '2026-10-21', expires: '2026-10-22',
        // The reversed invoice settles nothing; the draft one is not read.
        total: 250.5, settled: 0, outstanding: 250.5,
        lines: [{ name: 'Borders', qty: 1.5, total: 250.5 }]
      },
      {
        id: 103, ref: 'S00103', kind: 'quotation', customer: 'Hassan Cafe', salesperson: null,
        date: '2026-10-19', expires: null,
        // The draft-entry payment of 300 counts once, though the invoice is
        // reached both by id and by origin.
        total: 800, settled: 300, outstanding: 500,
        lines: [{ name: 'Racks', qty: 4, total: 800 }]
      }
    ]
  });
});

test('the_reads_are_the_filtered_json2_reads_and_nothing_else', async t => {
  const { odoo, reader } = await fakeAndReader(t);
  await buildOpenQuotations(reader, NOW);
  const calls = odoo.calls();
  assert.deepEqual(calls.map(c => `${c.model}.${c.method}`), [
    'sale.order.search_read',
    'account.move.search_read',
    'pos.order.line.search_read',
    'pos.order.search_read',
    'account.payment.search_read',
    'sale.order.line.read'
  ]);
  for (const c of calls) {
    assert.equal(c.verb, 'POST');
    assert.equal(c.headers.authorization, `bearer ${ODOO_KEY}`);
    assert.equal(c.headers['x-odoo-database'], ODOO_DB);
  }
  assert.deepEqual(calls[0].body.domain,
    [['state', 'in', ['draft', 'sent']], ['amount_total', '>', 0]],
    'quotations only, and never an empty one');
  // POS lines only for these quotations' lines, never every linked till line.
  assert.deepEqual(calls[2].body.domain,
    [['sale_order_line_id', 'in', [1011, 1012, 1013, 1021, 1031, 1041]]]);
  // Lines are read only for the quotations that made the cut.
  assert.deepEqual(calls[5].body.ids, [1011, 1012, 1013, 1031, 1041]);
});

test('the_reader_refuses_anything_but_search_read_and_read_before_the_wire', async t => {
  const { odoo, reader } = await fakeAndReader(t);
  for (const method of ['write', 'create', 'unlink', 'action_confirm', 'search', '__proto__']) {
    assert.deepEqual(await reader.call('sale.order', method, [101], {}),
      { ok: false, code: 'odoo_method_not_allowed' });
  }
  assert.deepEqual(await reader.call('sale.order/../res.users', 'read', [1], {}),
    { ok: false, code: 'odoo_method_not_allowed' });
  assert.equal(odoo.calls().length, 0, 'nothing reached the fake');
});

test('a_failed_read_builds_nothing_and_says_why_in_a_closed_code', async t => {
  const { odoo, reader } = await fakeAndReader(t);

  odoo.fail('account.payment');
  assert.deepEqual(await buildOpenQuotations(reader, NOW), { ok: false, code: 'odoo_http_500' });
  odoo.heal();

  odoo.set('ratelimit');
  assert.deepEqual(await buildOpenQuotations(reader, NOW), { ok: false, code: 'odoo_rate_limited' });
  odoo.set('garbage');
  assert.deepEqual(await buildOpenQuotations(reader, NOW), { ok: false, code: 'odoo_bad_answer' });
  odoo.heal();

  const wrongKey = makeJson2Reader({ ODOO_URL: odoo.url, ODOO_DB, ODOO_API_KEY: 'nope' });
  assert.deepEqual(await buildOpenQuotations(wrongKey, NOW), { ok: false, code: 'odoo_http_401' });

  const nowhere = makeJson2Reader({ ODOO_URL: 'http://127.0.0.1:9', ODOO_DB, ODOO_API_KEY: ODOO_KEY });
  assert.deepEqual(await buildOpenQuotations(nowhere, NOW), { ok: false, code: 'odoo_unreachable' });
});

test('no_open_quotation_is_an_empty_feed_not_a_failure', async t => {
  const records = quotationRecords();
  records['sale.order'] = records['sale.order'].filter(o => o.state !== 'draft' && o.id !== 104);
  const { odoo, reader } = await fakeAndReader(t, records);
  const built = await buildOpenQuotations(reader, NOW);
  assert.equal(built.ok, true);
  assert.equal(built.value.openCount, 0);
  assert.equal(built.value.openTotal, 0);
  assert.deepEqual(built.value.days, []);
  assert.deepEqual(built.value.quotations, []);
  assert.ok(!odoo.calls().some(c => c.method === 'read'), 'no lines to read, so no read');
});
