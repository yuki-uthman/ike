// The HTTP boundary: the only module that knows Request, Response, headers and
// status codes (decision 17). Every answer is JSON, and `saved: true` occurs in
// exactly one place — the 200 answer to a save, built only after the store has
// already awaited the entry and its receipt.
//
// Since D27 the Worker also SERVES the page (the static-assets binding, public/),
// so the page and this boundary share one origin. There are no CORS headers any
// more: nothing is meant to call this from another origin. Two gates stand in
// front of every route this Worker runs for, in this order (D27):
//   1. every non-GET must carry an Origin equal to the Worker's own, or 403;
//   2. every data route (/expenses/*, /quotations/*) needs a session, or 401.
// /auth/* is the only area open without a session.

import { isKnownCategory, listCategories } from './categories.mjs';
import { parseMvrToLaari } from './mvr.mjs';
import {
  checkPassword,
  clearedSessionCookie,
  endSession,
  recordSigninFailure,
  sessionCookie,
  sessionToken,
  sessionUser,
  signinFailures,
  signinWindow,
  signinWindowEnd,
  SIGNIN_LIMIT,
  startSession
} from './auth.mjs';
import { listDay, maldivesDate, saveEntry } from './save.mjs';
import { runReadBack } from './readback.mjs';
import { runSync, syncEntry } from './sync.mjs';

const RETRY_PATH = /^\/entries\/([^/]+)\/retry$/;

// The one place the area prefix is written (decision 17). This Worker is the
// Odoo layer, and expenses is one area inside it; the route vocabulary below
// ('/entries', '/categories', RETRY_PATH) stays area-free, so a later Odoo area
// is one more constant and one more gate rather than a rewrite of a handler.
const AREA = '/expenses';

// The areas that need a session (D27). Quotations has no route yet — it is the
// next issue's — but it is gated now, so its first route cannot be born open.
const SIGNED_IN_AREAS = [AREA, '/quotations'];
const AUTH = '/auth';

const RECEIPT_CAP_BYTES = 1000000; // the Worker's own declared cap (decision 16)
const BASE64 = /^[A-Za-z0-9+/]*={0,2}$/;

// A signed-in answer is about one person's session and must never be kept by a
// cache between the phone and this Worker.
function json(status, body, headers = {}) {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json', 'cache-control': 'no-store', ...headers }
  });
}

function refuse(status, error, extra = {}, headers = {}) {
  return json(status, { saved: false, error, ...extra }, headers);
}

function inArea(pathname, area) {
  return pathname === area || pathname.startsWith(area + '/');
}

/** Decode a base64 receipt strictly; null means "not base64". */
function decodeReceipt(value) {
  if (value === undefined || value === null || value === '') {
    return { bytes: null, length: 0 };
  }
  if (typeof value !== 'string') return null;
  if (value.length % 4 !== 0 || !BASE64.test(value)) return null;
  let binary;
  try {
    binary = atob(value);
  } catch {
    return null;
  }
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
  return { bytes, length: bytes.length };
}

const DESCRIPTION_LIMIT_CHARS = 200; // D19's limit, in Unicode code points

/**
 * D19's description, normalised: absent, null or '' is "none" and becomes '';
 * a string is trimmed and every run of whitespace collapsed to one space. null
 * means "not text at all", which the caller refuses. The limit is applied by the
 * caller, after this, so text only normalisation brings under 200 is kept.
 */
function normaliseDescription(value) {
  if (value === undefined || value === null) return '';
  if (typeof value !== 'string') return null;
  return value.trim().replace(/\s+/g, ' ');
}

async function readJsonObject(request) {
  let body;
  try {
    body = await request.json();
  } catch {
    return null;
  }
  return body !== null && typeof body === 'object' && !Array.isArray(body) ? body : null;
}

// ---- /auth: the one area open without a session (D23-D25) ----

/**
 * Sign-in. The failure budget is read BEFORE the password is checked, so an
 * exhausted window answers 429 even to the right password, and an unknown
 * username and a wrong password are one answer with one message (D25).
 */
async function handleSignin(request, env) {
  const source = request.headers.get('CF-Connecting-IP') || 'unknown';
  const now = Date.now();
  const index = signinWindow(now);

  if (await signinFailures(env.DB, source, index) >= SIGNIN_LIMIT) {
    const end = signinWindowEnd(index);
    const seconds = Math.max(0, Math.ceil((end.getTime() - now) / 1000));
    return refuse(429, 'too_many_attempts',
      { retry_at: end.toISOString() }, { 'Retry-After': String(seconds) });
  }

  const body = await readJsonObject(request);
  if (!body) return refuse(422, 'bad_json');

  const user = await checkPassword(env.DB, body.username, body.password);
  if (!user) {
    await recordSigninFailure(env.DB, source, index);
    return refuse(401, 'wrong_username_or_password');
  }
  const token = await startSession(env.DB, user.id, now);
  return json(200, { signed_in: true, username: user.username },
    { 'Set-Cookie': sessionCookie(token) });
}

/** Sign-out ends THIS phone's session only; `signout` in the script ends them all. */
async function handleSignout(request, env) {
  await endSession(env.DB, sessionToken(request.headers.get('Cookie')));
  return json(200, { signed_in: false }, { 'Set-Cookie': clearedSessionCookie() });
}

// ---- /expenses ----

async function handleSave(request, env) {
  const now = Date.now();

  const body = await readJsonObject(request);
  if (!body) return refuse(422, 'bad_json');

  if (!isKnownCategory(body.category)) {
    return refuse(422, 'unknown_category');
  }

  const amountLaari = parseMvrToLaari(body.amount);
  if (amountLaari === null) {
    return refuse(422, 'amount_not_positive_mvr');
  }

  const clientEntryId = typeof body.client_entry_id === 'string'
    ? body.client_entry_id.trim()
    : '';
  if (clientEntryId === '') {
    return refuse(422, 'invalid_client_entry_id');
  }

  // D19's optional description, judged after the category, the amount and the
  // id, so an over-long description cannot reveal which other field was also
  // wrong. Absent, null and whitespace-only all mean "none" and are stored as ''.
  const description = normaliseDescription(body.description);
  if (description === null) {
    return refuse(422, 'description_not_text');
  }
  if ([...description].length > DESCRIPTION_LIMIT_CHARS) {
    return refuse(422, 'description_too_long', { limit_chars: DESCRIPTION_LIMIT_CHARS });
  }

  const receipt = decodeReceipt(body.receipt);
  if (receipt === null) {
    return refuse(422, 'bad_receipt_base64');
  }
  if (receipt.length > RECEIPT_CAP_BYTES) {
    return refuse(413, 'receipt_too_large',
      { receipt_bytes: receipt.length, limit_bytes: RECEIPT_CAP_BYTES });
  }

  let entry;
  try {
    entry = await saveEntry(env.DB, {
      client_entry_id: clientEntryId,
      amount_laari: amountLaari,
      category: body.category,
      description,
      receipt: receipt.bytes,
      entry_date: maldivesDate(now),
      saved_at: new Date(now).toISOString()
    });
  } catch (err) {
    return refuse(503, 'not_stored', { detail: String(err && err.message || err) });
  }

  // Nothing above was deferred: the row is durable before this is written.
  return json(200, { saved: true, entry });
}

async function handleDay(url, env) {
  const asked = url.searchParams.get('date');
  const date = asked && /^\d{4}-\d{2}-\d{2}$/.test(asked) ? asked : maldivesDate();
  try {
    const entries = await listDay(env.DB, date);
    return json(200, { date, entries });
  } catch (err) {
    return refuse(503, 'not_read', { detail: String(err && err.message || err) });
  }
}

/**
 * D12's Retry button. It creates no new data and discloses nothing beyond what
 * GET /entries already serves; like every data route it needs a session and,
 * being a POST, the Worker's own Origin. `saved: true` still occurs in exactly
 * one place: the save answer.
 */
async function handleRetry(clientEntryId, env) {
  let entry;
  try {
    entry = await syncEntry(env, clientEntryId);
  } catch (err) {
    return refuse(503, 'not_read', { detail: String(err && err.message || err) });
  }
  if (!entry) return refuse(404, 'not_found');
  return json(200, { entry });
}

/** The data routes, reached only past both gates. */
async function route(request, url, env) {
  if (!inArea(url.pathname, AREA)) return refuse(404, 'not_found');
  const path = url.pathname.slice(AREA.length);

  if (request.method === 'GET' && path === '/categories') {
    return json(200, { categories: listCategories() });
  }
  if (request.method === 'GET' && path === '/entries') {
    return handleDay(url, env);
  }
  if (request.method === 'POST' && path === '/entries') {
    return handleSave(request, env);
  }
  if (request.method === 'POST') {
    const retry = RETRY_PATH.exec(path);
    if (retry) return handleRetry(decodeURIComponent(retry[1]), env);
  }
  return refuse(404, 'not_found');
}

export default {
  async fetch(request, env) {
    const url = new URL(request.url);

    // GATE 1 (D27), decided before any body is read, on every path. A browser
    // sends Origin on every cross-origin request and on every same-origin
    // non-GET, so a non-GET without OUR origin is a forged or foreign call.
    // Reads are left to gate 2: a same-origin GET or navigation carries no
    // Origin at all.
    const reads = request.method === 'GET' || request.method === 'HEAD';
    if (!reads && request.headers.get('Origin') !== url.origin) {
      return refuse(403, 'origin_not_allowed');
    }

    if (inArea(url.pathname, AUTH)) {
      const route = url.pathname.slice(AUTH.length);
      if (request.method === 'POST' && route === '/signin') return handleSignin(request, env);
      if (request.method === 'POST' && route === '/signout') return handleSignout(request, env);
      if (request.method === 'GET' && route === '/session') {
        const user = await sessionUser(env.DB, sessionToken(request.headers.get('Cookie')),
          Date.now());
        return user
          ? json(200, { signed_in: true, username: user.username })
          : refuse(401, 'signed_out');
      }
      return refuse(404, 'not_found');
    }

    // Everything else this layer serves lives under an area prefix. An unknown
    // path is the plain 404; with the static-assets binding in front, the
    // Worker only sees a path here when no page file answered it.
    if (!SIGNED_IN_AREAS.some(area => inArea(url.pathname, area))) {
      return refuse(404, 'not_found');
    }

    // GATE 2 (D27): every data route needs a live session.
    const token = sessionToken(request.headers.get('Cookie'));
    const user = await sessionUser(env.DB, token, Date.now());
    if (!user) return refuse(401, 'signed_out');
    const res = await route(request, url, env);
    // A renewed session hands the phone the new lifetime with this answer.
    if (user.renewed) res.headers.append('Set-Cookie', sessionCookie(token));
    return res;
  },

  // The cron driving port (D13). Both steps are AWAITED rather than deferred to
  // ctx.waitUntil: the schedule's whole job is that run.
  //
  // Sync first, read-back last (decision 14): the sync carries the owner's own
  // promise and must not queue behind a read that can take 10 s, and reading
  // last means an expense created in this very run is already in the id set, so
  // a chip is never a run stale. The read-back is attempted even when the sync
  // step failed — the two steps are independent, so one entry Odoo refuses
  // cannot freeze every chip.
  async scheduled(event, env) {
    try {
      await runSync(env);
    } catch (err) {
      // The read-back is owed its turn regardless.
    }
    await runReadBack(env);
  }
};
