// A fake Odoo speaking the JSON-2 API (POST /json/2/<model>/<method>) on
// loopback, for the quotations suites (D30). It answers the two read methods
// the Worker's reader knows, search_read and read, from records a suite seeds,
// and it logs every request it receives on any path, so a suite can also see
// that nothing else was called. No call through it can reach
// mrh-investment.odoo.com.
//
// Not an oracle and not a test (no `.test.mjs`).
//
// What it mirrors from the live Odoo (measured 2026-10-21): the bearer key in
// Authorization and the database in X-Odoo-Database; named arguments only
// (`domain` / `ids`, `fields`); a list of records as the answer, many2one as
// [id, name] and an unset value as false; `read` answering in the order of the
// ids asked. A domain operator it does not know is a 500, as an unsearchable
// field is on the live Odoo, so a suite cannot pass by accident.

import http from 'node:http';

export const ODOO_DB = 'mrh-investment';
export const ODOO_KEY = 'json2-key-must-never-escape-7c41d0';

function scalar(value) {
  return Array.isArray(value) && value.length === 2 ? value[0] : value;
}
function unset(value) {
  return value === false || value === null || value === undefined;
}
function same(got, want) {
  if (want === false) return unset(got);
  return !unset(got) && String(scalar(got)) === String(want);
}

/** Odoo's domain, for the operators the quotations build uses. */
export function matches(row, domain) {
  return domain.every(cond => {
    if (!Array.isArray(cond)) throw new Error('the fake takes only implicit-AND domains');
    const [field, op, want] = cond;
    const got = field === 'id' ? row.id : row[field];
    switch (op) {
      case '=': return same(got, want);
      case '!=': return !same(got, want);
      case 'in': return want.some(w => same(got, w));
      case 'not in': return !want.some(w => same(got, w));
      case '>': return !unset(got) && Number(got) > Number(want);
      default: throw new Error('operator not supported by the fake: ' + op);
    }
  });
}

function project(row, fields) {
  const out = { id: row.id };
  for (const f of fields || Object.keys(row)) out[f] = row[f] === undefined ? false : row[f];
  return out;
}

/**
 * Start the fake. `records` maps a model name to its rows, in the order
 * search_read returns them. `fail(model)` makes that model's reads answer 500.
 */
export async function startFakeOdooJson2(records = {}) {
  const state = { calls: [], records, failing: new Set(), mode: 'ok' };

  const send = (res, status, body) => {
    res.writeHead(status, { 'content-type': 'application/json' });
    res.end(typeof body === 'string' ? body : JSON.stringify(body));
  };

  const server = http.createServer(async (req, res) => {
    let raw = '';
    for await (const chunk of req) raw += chunk;
    const call = { path: req.url, verb: req.method, headers: req.headers, raw };
    state.calls.push(call);

    const m = /^\/json\/2\/([^/]+)\/([^/]+)$/.exec(req.url);
    if (!m || req.method !== 'POST') return send(res, 404, { error: 'no such endpoint' });
    const [, model, method] = m;
    Object.assign(call, { model, method });

    if (req.headers.authorization !== `bearer ${ODOO_KEY}`
      || req.headers['x-odoo-database'] !== ODOO_DB) {
      return send(res, 401, { name: 'werkzeug.exceptions.Unauthorized' });
    }
    if (state.mode === 'hang') return;             // never answers; socket held open
    if (state.mode === 'garbage') return send(res, 200, '{{not json');
    if (state.mode === 'ratelimit') return send(res, 429, 'Too Many Requests');
    if (state.failing.has(model)) return send(res, 500, { name: 'odoo.exceptions.UserError' });

    let body;
    try {
      body = JSON.parse(raw);
    } catch {
      return send(res, 400, { error: 'body is not JSON' });
    }
    call.body = body;
    const rows = state.records[model] || [];

    try {
      if (method === 'search_read') {
        return send(res, 200, rows.filter(r => matches(r, body.domain || []))
          .map(r => project(r, body.fields)));
      }
      if (method === 'read') {
        const byId = new Map(rows.map(r => [r.id, r]));
        return send(res, 200, (body.ids || []).filter(id => byId.has(id))
          .map(id => project(byId.get(id), body.fields)));
      }
    } catch (err) {
      return send(res, 500, { error: String(err.message) });
    }
    // Any other method is one the Worker promises never to call.
    return send(res, 403, { error: 'forbidden method ' + method });
  });

  await new Promise(r => server.listen(0, '127.0.0.1', r));
  return {
    url: 'http://127.0.0.1:' + server.address().port,
    state,
    set(mode) { state.mode = mode; },
    fail(model) { state.failing.add(model); },
    heal() { state.failing.clear(); state.mode = 'ok'; },
    calls: () => state.calls,
    close: () => new Promise(r => {
      server.closeAllConnections();
      server.close(r);
    })
  };
}

/**
 * The canned books the quotations suites seed (issue #8): every way money
 * reaches a quotation, and every record that must stay out of the feed.
 *
 *   S00101  draft  1000  part-settled at the till: 400 through a paid POS
 *                        order; a draft POS order's 300 is not money. Raised
 *                        20:30 UTC on the 20th = the 21st in the Maldives.
 *                        Carries a section line, which is not a product.
 *   S00102  sent    500  paid through an invoice found ONLY by its origin text
 *                        ("S00102, S00999"): closed, so not in the feed.
 *   S00103  draft   800  its invoice reads not_paid, but a payment registered
 *                        against it in a draft entry took 300. A payment over
 *                        two invoices and a draft payment do not count. The
 *                        invoice is reached by id AND by origin: counted once.
 *   S00104  sent  250.5  its only invoice was reversed, and a second invoice is
 *                        still a draft: neither settles anything.
 *   S00105  draft     0  an empty quotation: never listed.
 *   S00106  cancel  300  cancelled: never listed.
 *   S00107  sale    900  a confirmed order: quotations only, never listed.
 */
export function quotationRecords() {
  const partner = [7, 'Ali Stores'];
  const seller = [2, 'Mariyam'];
  return {
    'sale.order': [
      { id: 101, name: 'S00101', state: 'draft', amount_total: 1000, date_order: '2026-10-20 20:30:00', validity_date: '2026-10-25', partner_id: partner, user_id: seller, invoice_ids: [], order_line: [1011, 1012, 1013] },
      { id: 102, name: 'S00102', state: 'sent', amount_total: 500, date_order: '2026-10-18 06:00:00', validity_date: false, partner_id: partner, user_id: seller, invoice_ids: [], order_line: [1021] },
      { id: 103, name: 'S00103', state: 'draft', amount_total: 800, date_order: '2026-10-19 08:00:00', validity_date: false, partner_id: [8, 'Hassan Cafe'], user_id: false, invoice_ids: [702], order_line: [1031] },
      { id: 104, name: 'S00104', state: 'sent', amount_total: 250.5, date_order: '2026-10-21 03:00:00', validity_date: '2026-10-22', partner_id: false, user_id: seller, invoice_ids: [], order_line: [1041] },
      { id: 105, name: 'S00105', state: 'draft', amount_total: 0, date_order: '2026-10-21 05:00:00', validity_date: false, partner_id: partner, user_id: seller, invoice_ids: [], order_line: [] },
      { id: 106, name: 'S00106', state: 'cancel', amount_total: 300, date_order: '2026-10-17 05:00:00', validity_date: false, partner_id: partner, user_id: seller, invoice_ids: [], order_line: [1061] },
      { id: 107, name: 'S00107', state: 'sale', amount_total: 900, date_order: '2026-10-16 05:00:00', validity_date: false, partner_id: partner, user_id: seller, invoice_ids: [], order_line: [1071] }
    ],
    'sale.order.line': [
      { id: 1011, order_id: [101, 'S00101'], name: '[PP9] PAPER PLATE 9"', product_uom_qty: 10, price_total: 400, display_type: false },
      { id: 1012, order_id: [101, 'S00101'], name: 'Cups', product_uom_qty: 5, price_total: 600, display_type: false },
      { id: 1013, order_id: [101, 'S00101'], name: 'Kitchen', product_uom_qty: 0, price_total: 0, display_type: 'line_section' },
      { id: 1021, order_id: [102, 'S00102'], name: 'Trays', product_uom_qty: 2, price_total: 500, display_type: false },
      { id: 1031, order_id: [103, 'S00103'], name: 'Racks', product_uom_qty: 4, price_total: 800, display_type: false },
      { id: 1041, order_id: [104, 'S00104'], name: 'Borders', product_uom_qty: 1.5, price_total: 250.5, display_type: false },
      { id: 1061, order_id: [106, 'S00106'], name: 'Shelves', product_uom_qty: 1, price_total: 300, display_type: false },
      { id: 1071, order_id: [107, 'S00107'], name: 'Shelves', product_uom_qty: 3, price_total: 900, display_type: false }
    ],
    'account.move': [
      { id: 701, move_type: 'out_invoice', state: 'posted', payment_state: 'paid', amount_total: 500, amount_residual: 0, invoice_origin: 'S00102, S00999' },
      { id: 702, move_type: 'out_invoice', state: 'posted', payment_state: 'not_paid', amount_total: 800, amount_residual: 800, invoice_origin: 'S00103' },
      { id: 703, move_type: 'out_invoice', state: 'posted', payment_state: 'not_paid', amount_total: 100, amount_residual: 100, invoice_origin: false },
      { id: 704, move_type: 'out_invoice', state: 'posted', payment_state: 'reversed', amount_total: 250.5, amount_residual: 0, invoice_origin: 'S00104' },
      { id: 705, move_type: 'out_invoice', state: 'draft', payment_state: 'not_paid', amount_total: 250.5, amount_residual: 0, invoice_origin: 'S00104' }
    ],
    'pos.order.line': [
      { id: 9001, order_id: [501, 'Shop/0001'], sale_order_line_id: [1011, 'S00101 - PP9'], price_subtotal_incl: 400 },
      { id: 9002, order_id: [502, 'Shop/0002'], sale_order_line_id: [1012, 'S00101 - Cups'], price_subtotal_incl: 300 },
      { id: 9003, order_id: [503, 'Shop/0003'], sale_order_line_id: false, price_subtotal_incl: 75 }
    ],
    'pos.order': [
      { id: 501, state: 'paid' },
      { id: 502, state: 'draft' },
      { id: 503, state: 'paid' }
    ],
    'account.payment': [
      { id: 801, payment_type: 'inbound', partner_type: 'customer', state: 'in_process', amount: 300, reconciled_invoice_ids: [702] },
      { id: 802, payment_type: 'inbound', partner_type: 'customer', state: 'paid', amount: 999, reconciled_invoice_ids: [702, 703] },
      { id: 803, payment_type: 'inbound', partner_type: 'customer', state: 'draft', amount: 500, reconciled_invoice_ids: [702] },
      { id: 804, payment_type: 'outbound', partner_type: 'supplier', state: 'paid', amount: 800, reconciled_invoice_ids: [702] }
    ]
  };
}
