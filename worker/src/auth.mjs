// The sign-in rule (D22-D26): one shared account by default, its password kept
// only as a PBKDF2 hash, and every sign-in a stored session so `signout` ends
// every phone at once. Takes the D1 binding and plain values; knows nothing of
// Request, Response, headers or status codes (decision 17) — the cookie's
// spelling is the one exception, kept here so it has a single home.
//
// Dependency direction: index.mjs -> auth.mjs. scripts/user.mjs imports
// `hashPassword` and `PASSWORD_ROUNDS` from here, so the owner's script and the
// Worker can never hash a password two different ways.

/**
 * PBKDF2-SHA256 rounds for a NEW password (D23). Measured on a real Worker on
 * 2026-10-21: 10,000 rounds take 2-3 ms of CPU, inside the free plan's 10 ms.
 * Each user row stores its own rounds, so this can change without locking out
 * a password hashed under the old figure.
 */
export const PASSWORD_ROUNDS = 10000;

const HASH_BITS = 256;
const SALT_BYTES = 16;
const TOKEN_BYTES = 32;

const DAY_MS = 24 * 3600 * 1000;

/** A session lives 30 days after its last use (D24). */
export const SESSION_MS = 30 * DAY_MS;

/** A session's expiry is moved forward at most once a day, so a busy phone
 *  costs one write a day rather than one per request (D24). */
export const RENEW_AFTER_MS = DAY_MS;

/** Sign-in budget (D25): 5 failures per source IP per fixed 15-minute window. */
export const SIGNIN_WINDOW_MS = 15 * 60 * 1000;
export const SIGNIN_LIMIT = 5;

export const SESSION_COOKIE = 'ike_session';

const encoder = new TextEncoder();

function toHex(bytes) {
  return Array.from(bytes, b => b.toString(16).padStart(2, '0')).join('');
}

function fromHex(hex) {
  if (typeof hex !== 'string' || hex.length % 2 !== 0 || /[^0-9a-f]/i.test(hex)) {
    return null;
  }
  const out = new Uint8Array(hex.length / 2);
  for (let i = 0; i < out.length; i++) out[i] = parseInt(hex.slice(i * 2, i * 2 + 2), 16);
  return out;
}

function toBase64Url(bytes) {
  let binary = '';
  for (const b of bytes) binary += String.fromCharCode(b);
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

export function randomBytes(n) {
  const bytes = new Uint8Array(n);
  crypto.getRandomValues(bytes);
  return bytes;
}

export function newSalt() {
  return toHex(randomBytes(SALT_BYTES));
}

/** The password's PBKDF2-SHA256 hash, as hex, for a hex salt and a round count. */
export async function hashPassword(password, saltHex, rounds) {
  const salt = fromHex(saltHex);
  if (!salt) throw new Error('salt is not hex');
  const key = await crypto.subtle.importKey(
    'raw', encoder.encode(password), 'PBKDF2', false, ['deriveBits']);
  const bits = await crypto.subtle.deriveBits(
    { name: 'PBKDF2', hash: 'SHA-256', salt, iterations: rounds }, key, HASH_BITS);
  return toHex(new Uint8Array(bits));
}

async function sha256Hex(text) {
  const digest = await crypto.subtle.digest('SHA-256', encoder.encode(text));
  return toHex(new Uint8Array(digest));
}

function sameBytes(a, b) {
  if (!a || !b || a.byteLength !== b.byteLength) return false;
  // Workers' own constant-time comparison; Node (the owner's script) never
  // verifies, it only hashes.
  if (crypto.subtle.timingSafeEqual) return crypto.subtle.timingSafeEqual(a, b);
  let diff = 0;
  for (let i = 0; i < a.byteLength; i++) diff |= a[i] ^ b[i];
  return diff === 0;
}

/** The fixed sign-in window index for `at`. */
export function signinWindow(at) {
  return Math.floor(at / SIGNIN_WINDOW_MS);
}

/** When the window `index` ends — the moment the budget comes back. */
export function signinWindowEnd(index) {
  return new Date((index + 1) * SIGNIN_WINDOW_MS);
}

/** Failures already recorded for this source in this window. */
export async function signinFailures(db, source, index) {
  const row = await db
    .prepare('SELECT misses FROM signin_attempt WHERE source = ? AND "window" = ?')
    .bind(source, index)
    .first();
  return row ? Number(row.misses) : 0;
}

/** Only a failed sign-in consumes budget. */
export async function recordSigninFailure(db, source, index) {
  await db
    .prepare(
      'INSERT INTO signin_attempt (source, "window", misses) VALUES (?, ?, 1)'
      + ' ON CONFLICT(source, "window") DO UPDATE SET misses = misses + 1'
    )
    .bind(source, index)
    .run();
}

/**
 * The user `username` names, if `password` is theirs; otherwise null. An unknown
 * name, a disabled user and a wrong password are the same null, and the unknown
 * name still pays for one hash, so the answer's timing does not say which.
 */
export async function checkPassword(db, username, password) {
  if (typeof username !== 'string' || typeof password !== 'string') return null;
  const row = await db
    .prepare(
      'SELECT id, username, password_hash, salt, rounds FROM "user"'
      + ' WHERE username = ? AND disabled_at IS NULL'
    )
    .bind(username.trim())
    .first();
  const salt = row ? row.salt : '00'.repeat(SALT_BYTES);
  const rounds = row ? Number(row.rounds) : PASSWORD_ROUNDS;
  const given = fromHex(await hashPassword(password, salt, rounds));
  if (!row) return null;
  return sameBytes(given, fromHex(row.password_hash))
    ? { id: Number(row.id), username: row.username }
    : null;
}

/** A new session for `userId`; the token goes to the phone, only its hash is stored. */
export async function startSession(db, userId, now) {
  const token = toBase64Url(randomBytes(TOKEN_BYTES));
  const at = new Date(now).toISOString();
  await db
    .prepare(
      'INSERT INTO session (token_hash, user_id, created_at, last_used_at, expires_at)'
      + ' VALUES (?, ?, ?, ?, ?)'
    )
    .bind(await sha256Hex(token), userId, at, at, new Date(now + SESSION_MS).toISOString())
    .run();
  return token;
}

/**
 * The signed-in user a session token belongs to, or null when the token is
 * absent, unknown, expired, or its user is disabled. A session last used more
 * than a day ago is renewed: its expiry moves to 30 days from now, and `renewed`
 * says so, because the phone's cookie must be handed the new lifetime too or it
 * would lapse 30 days after sign-in however often it was used.
 */
export async function sessionUser(db, token, now) {
  if (typeof token !== 'string' || token === '') return null;
  const tokenHash = await sha256Hex(token);
  const row = await db
    .prepare(
      'SELECT s.user_id, s.last_used_at, s.expires_at, u.username FROM session s'
      + ' JOIN "user" u ON u.id = s.user_id'
      + ' WHERE s.token_hash = ? AND u.disabled_at IS NULL'
    )
    .bind(tokenHash)
    .first();
  if (!row) return null;
  if (Date.parse(row.expires_at) <= now) return null;
  const renewed = now - Date.parse(row.last_used_at) >= RENEW_AFTER_MS;
  if (renewed) {
    await db
      .prepare('UPDATE session SET last_used_at = ?, expires_at = ? WHERE token_hash = ?')
      .bind(new Date(now).toISOString(), new Date(now + SESSION_MS).toISOString(), tokenHash)
      .run();
  }
  return { id: Number(row.user_id), username: row.username, renewed };
}

/** Ends this one session (this phone). Ending an unknown token is not an error. */
export async function endSession(db, token) {
  if (typeof token !== 'string' || token === '') return;
  await db
    .prepare('DELETE FROM session WHERE token_hash = ?')
    .bind(await sha256Hex(token))
    .run();
}

/** The session token a Cookie header carries, or null. */
export function sessionToken(cookieHeader) {
  if (!cookieHeader) return null;
  for (const part of cookieHeader.split(';')) {
    const eq = part.indexOf('=');
    if (eq < 0) continue;
    if (part.slice(0, eq).trim() === SESSION_COOKIE) return part.slice(eq + 1).trim();
  }
  return null;
}

/** The Set-Cookie value that hands a phone its session (D24). */
export function sessionCookie(token) {
  return `${SESSION_COOKIE}=${token}; HttpOnly; Secure; SameSite=Strict; Path=/;`
    + ` Max-Age=${Math.floor(SESSION_MS / 1000)}`;
}

/** The Set-Cookie value that takes it away again. */
export function clearedSessionCookie() {
  return `${SESSION_COOKIE}=; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=0`;
}
