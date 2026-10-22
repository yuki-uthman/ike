// The one way the suites start the app since D27: a real workerd (miniflare) on
// a loopback socket that serves BOTH the page (the static-assets binding over
// public/) and the Worker, exactly as Cloudflare does — the Worker runs first for
// the paths worker/wrangler.json names in `assets.run_worker_first`, and every
// other path is a page file. Config facts are read from wrangler.json, the
// config SSOT, never retyped here.
//
// Not an oracle and not a test (no `.test.mjs`): `node --test 'tests/*.test.mjs'`
// does not run it. Each suite still states its own claims; this module only
// gives them the app, a signed-in caller and the D1 binding.

import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { Miniflare } from 'miniflare';

import { addUserSql } from '../../worker/scripts/user.mjs';

export const REPO = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
export const WORKER = path.join(REPO, 'worker');

/** The shared account every suite signs in with, unless it says otherwise. */
export const USERNAME = 'manager';
export const PASSWORD = 'correct-horse-battery-staple-ike';

export async function readConfig() {
  return JSON.parse(await fs.readFile(path.join(WORKER, 'wrangler.json'), 'utf8'));
}

/** Every migration, in file-name order, as `wrangler d1 migrations apply` runs them. */
export async function applySchema(db, cfg) {
  const dir = path.join(WORKER, cfg.d1_databases[0].migrations_dir);
  const files = (await fs.readdir(dir)).filter(f => f.endsWith('.sql')).sort();
  const sql = (await Promise.all(files.map(f => fs.readFile(path.join(dir, f), 'utf8')))).join('\n');
  const statements = sql
    .split('\n').map(l => l.replace(/--.*$/, '')).join('\n')
    .split(';').map(s => s.trim()).filter(Boolean);
  await db.batch(statements.map(s => db.prepare(s)));
}

/** The owner's own `user.mjs add` statement, run against this D1. */
export async function addUser(db, username = USERNAME, password = PASSWORD, options = {}) {
  await db.exec((await addUserSql(username, password, options)).replace(/\s+/g, ' '));
}

/**
 * Start the app. `bindings` adds to (or overrides) wrangler.json's vars — the
 * Odoo suites point ODOO_URL at their fake and add ODOO_API_KEY. `schema: false`
 * leaves the D1 empty (a suite that wants to see the missing-schema answer).
 * `user: false` creates no account.
 */
export async function startApp({ bindings = {}, persist, schema = true, user = true } = {}) {
  const cfg = await readConfig();
  const binding = cfg.d1_databases[0].binding;
  const mf = new Miniflare({
    scriptPath: path.join(WORKER, cfg.main),
    modules: true,
    modulesRoot: WORKER,
    compatibilityDate: cfg.compatibility_date,
    compatibilityFlags: cfg.compatibility_flags || [],
    d1Databases: { [binding]: cfg.d1_databases[0].database_id || binding },
    d1Persist: persist,
    bindings: { ...(cfg.vars || {}), ...bindings },
    assets: {
      directory: path.resolve(WORKER, cfg.assets.directory),
      routerConfig: {
        has_user_worker: true,
        static_routing: { user_worker: cfg.assets.run_worker_first }
      }
    },
    port: 0
  });
  const url = await mf.ready;
  const db = await mf.getD1Database(binding);
  if (schema) await applySchema(db, cfg);
  if (schema && user) await addUser(db);
  const origin = url.origin;

  /**
   * One HTTP call, as the page makes it. A non-GET carries this app's own
   * Origin (a browser adds it to every same-origin POST) unless `origin` says
   * otherwise (null = send none). `cookie` is the session cookie, or null.
   */
  const call = async (method, pathname, { origin: from, body, ip = '127.0.0.1', raw,
    cookie = null, headers: extra = {} } = {}) => {
    const headers = { 'CF-Connecting-IP': ip, ...extra };
    const reads = method === 'GET' || method === 'HEAD';
    const sendOrigin = from === undefined ? (reads ? null : origin) : from;
    if (sendOrigin !== null) headers.Origin = sendOrigin;
    if (cookie) headers.Cookie = cookie;
    if (body !== undefined || raw !== undefined) headers['content-type'] = 'application/json';
    const res = await fetch(origin + pathname, {
      method,
      headers,
      redirect: 'manual',
      body: raw !== undefined ? raw : (body === undefined ? undefined : JSON.stringify(body))
    });
    const text = await res.text();
    let json = null;
    try { json = JSON.parse(text); } catch { /* the caller asserts */ }
    return { res, json, text };
  };

  /** Sign in and return the session cookie ('ike_session=<token>'). */
  const signIn = async (username = USERNAME, password = PASSWORD, opts = {}) => {
    const r = await call('POST', '/auth/signin', { body: { username, password }, ...opts });
    if (r.res.status !== 200) {
      throw new Error(`sign-in as ${username} answered ${r.res.status} ${r.text}`);
    }
    return sessionCookieOf(r.res);
  };

  return { mf, db, cfg, origin, call, signIn, close: () => mf.dispose() };
}

/** The 'name=value' of the session cookie a response sets, or null. */
export function sessionCookieOf(res) {
  const all = typeof res.headers.getSetCookie === 'function'
    ? res.headers.getSetCookie()
    : [res.headers.get('set-cookie')].filter(Boolean);
  const set = all.find(c => c.startsWith('ike_session='));
  return set ? set.split(';')[0] : null;
}

/**
 * A browser context already signed in to `app`, the way a phone that signed in
 * yesterday opens the app: it holds the session cookie, so the page opens on
 * the app and not on the sign-in screen.
 */
export async function signedInContext(browser, app, contextOptions = {}) {
  const cookie = await app.signIn();
  const [name, value] = cookie.split('=');
  const context = await browser.newContext(contextOptions);
  await context.addCookies([{
    name, value, url: app.origin, httpOnly: true, secure: true, sameSite: 'Strict'
  }]);
  return context;
}
