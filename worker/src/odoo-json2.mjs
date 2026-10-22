// The driven port for Odoo's JSON-2 API (POST /json/2/<model>/<method>), and a
// READ-ONLY one by construction: it knows two methods, search_read and read,
// and refuses every other name before anything reaches the wire, as readback.mjs
// does for the XML-RPC client (D30). The XML-RPC write path in odoo.mjs is a
// separate module and this one shares nothing with it.
//
// JSON-2 rather than XML-RPC because the free plan's 10 ms CPU limit leaves no
// room for decoding XML in V8: the full quotation read takes ~10 ms as XML and
// ~4 ms as JSON (measured on a real Worker, 2026-10-21).
//
// Every answer is {ok: true, value} or {ok: false, code}, the code drawn from a
// closed set — odoo_method_not_allowed, odoo_unreachable, odoo_timeout,
// odoo_rate_limited, odoo_http_<status>, odoo_bad_answer — so no Odoo text and
// no credential can reach a D1 column, a response body or a log line.

const TIMEOUT_MS = 10000;

// JSON-2 takes named arguments only. Each allowed method, and the name its
// first argument travels under.
const READS = { search_read: 'domain', read: 'ids' };

const MODEL = /^[a-z0-9_.]+$/;

function networkCode(err) {
  const name = err && err.name;
  return name === 'TimeoutError' || name === 'AbortError'
    ? 'odoo_timeout'
    : 'odoo_unreachable';
}

/**
 * The reader. The key is read from env on every call and never kept in a
 * module-level variable; there is no session to open, the key is the bearer.
 */
export function makeJson2Reader(env) {
  const base = String(env.ODOO_URL || '').replace(/\/+$/, '');

  return {
    /** call('sale.order', 'search_read', domain, {fields}) or call(m, 'read', ids, {fields}). */
    async call(model, method, first, named = {}) {
      if (!Object.prototype.hasOwnProperty.call(READS, method) || !MODEL.test(model)) {
        return { ok: false, code: 'odoo_method_not_allowed' };
      }

      let res;
      try {
        res = await fetch(`${base}/json/2/${model}/${method}`, {
          method: 'POST',
          headers: {
            'content-type': 'application/json',
            authorization: `bearer ${env.ODOO_API_KEY}`,
            'x-odoo-database': String(env.ODOO_DB || '')
          },
          body: JSON.stringify({ ...named, [READS[method]]: first }),
          signal: AbortSignal.timeout(TIMEOUT_MS)
        });
      } catch (err) {
        return { ok: false, code: networkCode(err) };
      }

      if (res.status === 429) return { ok: false, code: 'odoo_rate_limited' };
      if (!res.ok) return { ok: false, code: `odoo_http_${res.status}` };

      let value;
      try {
        value = await res.json();
      } catch (err) {
        return { ok: false, code: err && err.name === 'SyntaxError' ? 'odoo_bad_answer' : networkCode(err) };
      }
      // Both methods answer a list of records; anything else is not an answer.
      if (!Array.isArray(value)) return { ok: false, code: 'odoo_bad_answer' };
      return { ok: true, value };
    }
  };
}
