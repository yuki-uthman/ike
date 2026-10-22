-- D22-D25: the sign-in. The production D1 has already applied 0001 and 0002, so
-- the new tables arrive through this migration and neither earlier file is
-- edited. Nothing is added to `entry`: there is one shared account and no
-- record of who posted (D22). The `pin_attempt` table stays, unused, rather
-- than being dropped by a destructive migration.

-- The accounts. One shared username is the plan, but the table takes more names
-- later without a rebuild. The password is never stored, only its PBKDF2-SHA256
-- hash, the salt it was made with and its own round count, so a later change of
-- rounds cannot lock out a password hashed under the old figure. disabled_at
-- set means the account signs nobody in.
CREATE TABLE IF NOT EXISTS "user" (
  id            INTEGER PRIMARY KEY,
  username      TEXT NOT NULL UNIQUE,
  password_hash TEXT NOT NULL,
  salt          TEXT NOT NULL,
  rounds        INTEGER NOT NULL,
  created_at    TEXT NOT NULL,
  disabled_at   TEXT
);

-- One row per signed-in phone. Only the SHA-256 of the cookie's token is kept,
-- so a copy of this table signs nobody in. A session expires 30 days after its
-- last use (D24).
CREATE TABLE IF NOT EXISTS session (
  token_hash   TEXT PRIMARY KEY,
  user_id      INTEGER NOT NULL REFERENCES "user" (id),
  created_at   TEXT NOT NULL,
  last_used_at TEXT NOT NULL,
  expires_at   TEXT NOT NULL
);

-- `signout` ends every session of one user, through this index.
CREATE INDEX IF NOT EXISTS session_by_user ON session (user_id);

-- The failed sign-in counter (D25): misses per source per fixed 15-minute
-- window. It holds no username and no password.
CREATE TABLE IF NOT EXISTS signin_attempt (
  source   TEXT NOT NULL,
  "window" INTEGER NOT NULL,
  misses   INTEGER NOT NULL,
  PRIMARY KEY (source, "window")
);
