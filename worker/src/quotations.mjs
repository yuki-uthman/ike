// The open-quotations feed (D29, D31): built from Odoo every 15 minutes by the
// second cron, kept in D1 as one JSON snapshot, and served to the Quotations
// pane by GET /quotations/open.
//
// The rules are ike-data's scripts/fetch_quotations.py, ported for quotations
// only, so the feed and quotations.json agree. Read its header docstring for
// why `invoice_status` and `invoice_ids` cannot be trusted here. In short, a
// quotation's settled value is computed, never read:
//
//   settled = paid invoice value  (invoices found by invoice_ids AND by the
//                                  order name in invoice_origin, reversed ones
//                                  skipped, each counted as max(booked,
//                                  draft-entry payments) capped at its total)
//           + POS-settled value   (pos.order.line.sale_order_line_id, only
//                                  for POS orders that are money)
//
// and a quotation is open while amount_total - settled is above one cent.
//
// The arithmetic matches Python's on purpose: the same additions in the same
// order, round() as Python rounds (pyRound2) and sum() as Python 3.12 sums
// (pySum). This runs inside the free plan's 10 ms CPU limit, so no
// localeCompare and no Intl: plain < and > only.
//
// Dependency direction: index.mjs -> quotations.mjs -> {odoo-json2.mjs,
// save.mjs}. It never imports odoo.mjs, so it cannot reach the write path.

import { makeJson2Reader } from './odoo-json2.mjs';
import { maldivesDate } from './save.mjs';

export const FEED_NAME = 'quotations';

// A cent of float noise must not keep a fully settled quotation on the list.
const EPSILON = 0.01;
const DAY_MS = 86400000;

// Odoo has renamed payment states across versions, so exclude what is
// definitely not money rather than listing what is (odoo_payments.py).
const NOT_RECEIVED_STATES = ['draft', 'cancel', 'canceled', 'cancelled'];
const POS_NOT_MONEY = ['draft', 'cancel'];

/**
 * Python's round(x, 2). toFixed already rounds the exact binary value, as
 * Python does, so the two differ only on an exact binary tie: a value whose
 * third decimal is exactly 5, which in binary means an odd multiple of 1/8.
 * Python rounds those half to even, toFixed away from zero.
 */
export function pyRound2(x) {
  if (Number.isInteger(x * 8) && !Number.isInteger(x * 4)) {
    const low = Math.floor(x * 100);
    return (low % 2 === 0 ? low : low + 1) / 100;
  }
  return Number(x.toFixed(2));
}

/** Python 3.12's sum() over floats: Neumaier's compensated summation. */
export function pySum(values) {
  let total = 0;
  let c = 0;
  for (const x of values) {
    const t = total + x;
    if (Math.abs(total) >= Math.abs(x)) c += (total - t) + x;
    else c += (x - t) + total;
    total = t;
  }
  return c && Number.isFinite(c) ? total + c : total;
}

/** invoice_origin is free text and may carry several order names. */
export function splitOrigins(origin) {
  if (!origin) return [];
  return String(origin).split(/[,\s]+/).filter(Boolean);
}

/** Odoo returns a many2one as [id, display_name], or false when unset. */
function relId(value) {
  return Array.isArray(value) && value.length === 2 ? value[0] : null;
}
function relName(value) {
  return Array.isArray(value) && value.length === 2 ? value[1] : null;
}

/** Odoo datetimes are naive UTC strings; the day is the Maldives day (UTC+5). */
function maldivesDay(odooDatetime) {
  if (!odooDatetime) return null;
  const at = Date.parse(String(odooDatetime).replace(' ', 'T') + 'Z');
  return Number.isNaN(at) ? null : maldivesDate(at);
}

function addTo(map, key, amount) {
  map.set(key, (map.has(key) ? map.get(key) : 0) + amount);
}

/**
 * Read Odoo and build the feed. {ok: true, value: feed} or the first failed
 * read's {ok: false, code}: a run that could not read everything builds
 * nothing, so it can never publish a partial list.
 */
export async function buildOpenQuotations(odoo, now = Date.now()) {
  const orders = await odoo.call('sale.order', 'search_read',
    [['state', 'in', ['draft', 'sent']], ['amount_total', '>', 0]],
    { fields: ['name', 'state', 'amount_total', 'date_order', 'validity_date',
      'partner_id', 'user_id', 'invoice_ids', 'order_line'] });
  if (!orders.ok) return orders;

  // --- Channel 1: accounting invoices, reached by id AND by origin name ---
  const moves = await odoo.call('account.move', 'search_read',
    [['move_type', 'in', ['out_invoice', 'out_receipt']], ['state', '=', 'posted']],
    { fields: ['move_type', 'payment_state', 'amount_total', 'amount_residual', 'invoice_origin'] });
  if (!moves.ok) return moves;
  const moveById = new Map();
  const movesByOrigin = new Map();
  for (const m of moves.value) {
    moveById.set(m.id, m);
    for (const token of splitOrigins(m.invoice_origin)) {
      if (!movesByOrigin.has(token)) movesByOrigin.set(token, []);
      movesByOrigin.get(token).push(m);
    }
  }

  // --- Channel 2: the POS counter, settling order lines directly ---
  // Only the lines that settle one of these quotations: fetch_quotations.py
  // reads every linked POS line (it also lists confirmed orders), and the
  // ones pointing at a confirmed order would only be read to be dropped.
  const lineToOrder = new Map();
  for (const o of orders.value) for (const lid of o.order_line || []) lineToOrder.set(lid, o.id);
  let posLines = { ok: true, value: [] };
  if (lineToOrder.size) {
    posLines = await odoo.call('pos.order.line', 'search_read',
      [['sale_order_line_id', 'in', [...lineToOrder.keys()]]],
      { fields: ['order_id', 'price_subtotal_incl', 'sale_order_line_id'] });
    if (!posLines.ok) return posLines;
  }
  // Only the POS orders those lines point at, never every till receipt.
  const posOrderIds = [...new Set(posLines.value.map(pl => relId(pl.order_id)))]
    .filter(id => id !== null)
    .sort((a, b) => a - b);
  const posState = new Map();
  if (posOrderIds.length) {
    const posOrders = await odoo.call('pos.order', 'search_read',
      [['id', 'in', posOrderIds]], { fields: ['state'] });
    if (!posOrders.ok) return posOrders;
    for (const p of posOrders.value) posState.set(p.id, p.state);
  }
  const posSettled = new Map();
  for (const pl of posLines.value) {
    const id = relId(pl.order_id);
    if (!posState.has(id) || POS_NOT_MONEY.includes(posState.get(id))) continue;
    const orderId = lineToOrder.get(relId(pl.sale_order_line_id));
    if (orderId) addTo(posSettled, orderId, pl.price_subtotal_incl);
  }

  // --- Channel 3: payments Odoo has not yet booked onto the invoice ---
  // Only a payment against a single invoice counts: with several there is no
  // telling how the amount splits. Filtered here because
  // reconciled_invoice_ids is computed and not searchable.
  const payments = await odoo.call('account.payment', 'search_read',
    [['payment_type', '=', 'inbound'], ['partner_type', '=', 'customer'],
      ['state', 'not in', NOT_RECEIVED_STATES]],
    { fields: ['amount', 'reconciled_invoice_ids'] });
  if (!payments.ok) return payments;
  const paidByMove = new Map();
  for (const pay of payments.value) {
    const linked = pay.reconciled_invoice_ids || [];
    if (linked.length === 1) addTo(paidByMove, linked[0], pay.amount);
  }

  function invoiceSettled(order) {
    const seen = new Set();
    let total = 0;
    const candidates = (order.invoice_ids || []).filter(id => moveById.has(id)).map(id => moveById.get(id))
      .concat(movesByOrigin.get(order.name) || []);
    for (const m of candidates) {
      if (seen.has(m.id) || m.payment_state === 'reversed') continue;
      seen.add(m.id);
      // The larger of the two, never the sum: once Odoo posts the entry both
      // describe the same money. Capped at the invoice total.
      const booked = m.amount_total - m.amount_residual;
      total += Math.min(m.amount_total, Math.max(booked, paidByMove.has(m.id) ? paidByMove.get(m.id) : 0));
    }
    return total;
  }

  const open = [];
  for (const o of orders.value) {
    const settled = invoiceSettled(o) + (posSettled.has(o.id) ? posSettled.get(o.id) : 0);
    const outstanding = pyRound2(o.amount_total - settled);
    if (outstanding > EPSILON) open.push({ o, settled: pyRound2(settled), outstanding });
  }

  // --- Lines, for the chevron detail. Only for quotations that made the cut. ---
  const lineIds = open.flatMap(({ o }) => o.order_line || []);
  const linesByOrder = new Map();
  if (lineIds.length) {
    const lines = await odoo.call('sale.order.line', 'read', lineIds,
      { fields: ['order_id', 'name', 'product_uom_qty', 'price_total', 'display_type'] });
    if (!lines.ok) return lines;
    for (const l of lines.value) {
      if (l.display_type) continue;
      const id = relId(l.order_id);
      if (!linesByOrder.has(id)) linesByOrder.set(id, []);
      linesByOrder.get(id).push({ name: l.name, qty: l.product_uom_qty, total: pyRound2(l.price_total) });
    }
  }

  const quotations = open.map(({ o, settled, outstanding }) => ({
    id: o.id,
    ref: o.name,
    kind: 'quotation',
    customer: relName(o.partner_id),
    salesperson: relName(o.user_id),
    date: maldivesDay(o.date_order),
    expires: o.validity_date || null,
    total: pyRound2(o.amount_total),
    settled,
    outstanding,
    lines: linesByOrder.get(o.id) || []
  }));
  // Newest first, biggest first within a day; ties keep Odoo's order, as
  // Python's stable reverse sort does. The page re-sorts per view.
  quotations.sort((a, b) => {
    const da = a.date || '';
    const db = b.date || '';
    if (da !== db) return da < db ? 1 : -1;
    return b.outstanding - a.outstanding;
  });

  // --- Day totals for the chart, gaps filled from the oldest open day to
  // today, so the axis is a real timeline that shrinks as old ones settle. ---
  const perDay = new Map();
  for (const q of quotations) {
    if (!q.date) continue;
    if (!perDay.has(q.date)) perDay.set(q.date, { count: 0, outstanding: 0, total: 0 });
    const bucket = perDay.get(q.date);
    bucket.count += 1;
    bucket.outstanding += q.outstanding;
    bucket.total += q.total;
  }
  const days = [];
  if (perDay.size) {
    const today = maldivesDate(now);
    let first = null;
    for (const day of perDay.keys()) if (first === null || day < first) first = day;
    for (let at = Date.parse(first + 'T00:00:00Z'); ; at += DAY_MS) {
      const key = new Date(at).toISOString().slice(0, 10);
      if (key > today) break;
      const bucket = perDay.get(key) || { count: 0, outstanding: 0, total: 0 };
      days.push({
        date: key,
        count: bucket.count,
        outstanding: pyRound2(bucket.outstanding),
        total: pyRound2(bucket.total)
      });
    }
  }

  const openTotal = pyRound2(pySum(quotations.map(q => q.outstanding)));
  return {
    ok: true,
    value: {
      company: 'MRH Investment',
      currency: 'MVR',
      generatedAt: new Date(now).toISOString().slice(0, 19) + 'Z',
      openCount: quotations.length,
      openTotal,
      quotationCount: quotations.length,
      quotationTotal: openTotal,
      days,
      quotations
    }
  };
}

/** The stored snapshot's JSON text, or null before the first good build. */
export async function readFeed(db, name = FEED_NAME) {
  const row = await db.prepare('SELECT json FROM feed WHERE name = ?').bind(name).first();
  return row ? row.json : null;
}

async function writeFeed(db, name, feed) {
  await db.prepare(
    'INSERT INTO feed (name, json, generated_at) VALUES (?, ?, ?) '
    + 'ON CONFLICT (name) DO UPDATE SET json = excluded.json, generated_at = excluded.generated_at'
  ).bind(name, JSON.stringify(feed), feed.generatedAt).run();
}

/**
 * The scheduled build. The row is written only after a fully successful run,
 * so a failed run keeps the previous snapshot.
 */
export async function runQuotations(env, now = Date.now()) {
  const built = await buildOpenQuotations(makeJson2Reader(env), now);
  if (!built.ok) return built;
  await writeFeed(env.DB, FEED_NAME, built.value);
  return { ok: true, openCount: built.value.openCount };
}
