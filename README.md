# Ike Sales

A small, free dashboard of daily takings at MRH Investment — the money that
actually arrived each day, counted once each — with a follow-up list of open
quotations and a way for managers to record expenses. The page itself is
**static**: `public/index.html`, no secret, no build step. It reads its figures
live from [ike-data](https://github.com/yuki-uthman/ike-data), the shared Odoo
pipeline.

Since 2026-10 the page is served by the `odoo` Cloudflare Worker kept in this
repo, at **https://odoo.ike-mrh.workers.dev**, and the whole app sits behind
**one shared sign-in** (see below). The Worker also saves the Expenses pane's
entries.

## Three panes: Sales, Quotations and Expenses

The page is three panes side by side — **Sales** on the left (the default),
**Quotations** in the middle (the follow-up list that used to be its own repo,
`ike-quotations`) and **Expenses** on the right. Swipe left or right to move
between them, or tap a tab in the bar along the bottom. The bar's indicator follows the swipe as it
happens, so it always shows which pane you are on.

Two things follow from the swipe belonging to the pager:

- **Swiping no longer changes the day in Sales.** Pick a day from the pills or
  the bars, or use the left/right arrow keys on a keyboard.
- **A drag that starts on a chart or on the day pills scrolls that strip**, and
  stops there at its ends. Start the swipe anywhere else, or tap the tab.

Each pane keeps its own accent: teal for money banked, amber for money still
awaited, plum for money spent. The panes share one file but not their code —
every id and class in a pane is scoped to it (`s-…` / `q-…`,
`.pane-sales` / `.pane-quotes` / `.pane-expenses`).

The selected day is shared by all three panes: pick a day in one and the others
follow. (Quotations' **All** stays local to that pane.)

## One-time setup

Everything is deployed from `worker/` with the pinned wrangler (the owner does
this; nothing deploys on its own):

```
cd worker
npx wrangler@4.148.0 d1 migrations apply ike-odoo --remote
npx wrangler@4.148.0 deploy
```

`deploy` uploads the Worker **and** the page: `worker/wrangler.json` names
`../public` as the Worker's static assets, and runs the Worker first only for
`/auth/*`, `/expenses/*` and `/quotations/*`. Every other path is a page file.

Then create the shared account (see *Signing in*). Data freshness, the read-side
Odoo credential and the de-duplication logic live in
[ike-data](https://github.com/yuki-uthman/ike-data). GitHub Pages is no longer
used: turn it off (Settings → Pages → Source: None) once the Worker serves the
app.

## Signing in

The app opens on a sign-in screen. Sales, Quotations and Expenses appear only
after a username and password are accepted. There is **one shared account** for
all managers; nothing records who posted an expense.

- **Passwords are generated, never chosen**: five random words from the
  7,776-word list in `worker/scripts/`, joined by `-`. Only a PBKDF2-SHA256 hash
  (10,000 rounds) is stored.
- **Accounts are managed with one script**, which prints the SQL to run and
  touches no database itself:

  ```
  node worker/scripts/user.mjs add <name>       # new account, or a new password
  node worker/scripts/user.mjs signout <name>   # ends every phone's session
  node worker/scripts/user.mjs remove <name>    # ends the sessions, deletes the account
  npx wrangler@4.148.0 d1 execute ike-odoo --remote --command "<the printed SQL>"
  ```

  `add` prints the new password once. To take someone off: `add` (a new
  password) then `signout`, and give the new password to the others.
- **A sign-in lasts 30 days after the phone last used it.** Sign out (at the
  bottom of Expenses) ends that phone's session only.
- **Five wrong tries from one connection in 15 minutes** lock it out until the
  window ends; the screen says at what time (Maldives time).

What the sign-in does **not** hide: the Sales and Expenses totals still come from
ike-data's public files (see *Visibility*).

## Installing it on Android

The page is an installable web app. Open https://odoo.ike-mrh.workers.dev in
Chrome on the phone, sign in, tap the ⋮ menu, then **Install app** (or **Add to
Home screen**). It gets the Ike icon and opens full-screen with no browser bar.
An app installed from the old github.io address must be uninstalled first; its
"Not interested" marks and remembered views do not carry over.

- `public/manifest.webmanifest` names the app and lists the icons in
  `public/icons/`. The `maskable-*` icons are full-bleed so Android can cut them
  to any shape; the `icon-*` ones keep the rounded tile.
- `public/sw.js` exists to make the app installable and to open the shell with
  no signal. It never caches data: the JSON feeds, `/auth/`, `/expenses/` and
  `/quotations/` go straight to the network, and the page itself is fetched
  network-first.
- To change the icon, replace the files in `public/icons/` (192 and 512 px,
  plain and maskable) and keep the paths in the manifest.

## What a day's number means

**Money received that day**, not sales made that day — the same question
[ike-pos](https://github.com/yuki-uthman/ike-pos) (formerly ike-today) asks,
so the two dashboards can never disagree. A credit sale confirmed today does
not appear here until the payment lands; an old invoice settled today appears
here today.

The product list underneath is the products on whatever was paid for that
day. One caveat: a partial payment contributes its invoice's whole line set,
so in that case the products total more than the day's figure. MRH settles
invoices in full as a rule, so this is a stated edge rather than a routine
distortion.

A cheque, a card, or any other method that is neither cash nor transfer is
still money received, so it counts in this dashboard's total. ike-pos, which
has only a cash card and a transfer card to put it in, holds it out of both
and names it in a separate strip instead.

**This changed on 2026-09-16.** The dashboard used to count sales *made* each
day, paid or not. The full 60-day history was re-backfilled under the new
meaning at the same time, so the chart is comparable end to end.

## How it gets its data

`public/index.html` fetches
`https://raw.githubusercontent.com/yuki-uthman/ike-data/main/data/sales.json`
directly in the browser on every page load (`cache: 'no-store'`) — GitHub
serves raw file content with `Access-Control-Allow-Origin: *`, so this works
cross-repo with no server or API needed. See
[ike-data](https://github.com/yuki-uthman/ike-data)'s README for how that
file gets refreshed and what "Point of Sale" vs "Sales Orders" actually
counts.

## The Quotations pane

Every quotation that still has money owing on it — what hasn't closed yet, so
it can be chased. One card, **Quotations**: open quotations only (state
*Quotation* or *Quotation Sent*, total above zero). Confirmed sales orders are
not listed; the old **Unpaid orders** card was dropped on 2026-10-21. Above the
card, a bar per day showing how much still-open value was raised that day
(Maldives days). It opens on today; **All**, the pinned pill on the left, is
the whole pipeline.

**Still open** means any part of the value is unsettled, and a record leaves the
moment the last rufiyaa arrives — by invoice or at the POS. "Paid" is
*computed*, not read: `invoice_status` and `invoice_ids` both lie on this Odoo
instance, so the number behind each row is
`amount_total − (paid invoice value + POS-settled value)`. A part-paid
quotation shows its remaining balance, with the total beneath. The rules are
those of `ike-data/scripts/fetch_quotations.py` (read its header for why),
ported to `worker/src/quotations.mjs`; the two give the same figures.

**Not interested ×** (inside an expanded row) hides a dead lead and takes its
value out of the totals and the day's bar. It does **not** touch Odoo, and it
is remembered only in that browser's `localStorage` — dismiss on a laptop and it
is still there on a phone. A counter at the bottom of the pane lists what is
hidden and restores it.

The Worker builds this list itself: at 7, 22, 37 and 52 minutes past each
hour it reads Odoo (read-only, over Odoo's JSON-2 API) and keeps the result in
its database. The pane reads it from `GET /quotations/open` (signed in only) on
load, then again every 60 seconds and whenever the tab becomes visible. Open
rows are remembered across a refresh. A build that fails keeps the last good
list. Right after the first deploy there is no list yet, and the pane says
*"Quotations appear after the next refresh, within 15 minutes."* It is a live
snapshot with no history; its day span shrinks as old quotations settle.

ike-data still writes `quotations.json` (with the unpaid orders) on its own
schedule, but this page no longer reads it.

## The Expenses pane

The manager records a **bank-transfer expense** from a phone: pick a category,
enter the amount in MVR, optionally type a description and attach a receipt
photo, and save. The day's entries are listed under day pills
(oldest on the left, today on the right), each with a status chip.

What it does and does not do, by design:

- **It only creates a draft `hr.expense` in Odoo.** It never submits, approves,
  posts or pays. The accountant does that in Odoo.
- **Save first, send later.** The entry is stored in the Worker's own database
  immediately, then synced to Odoo. A failed send is retried on a schedule and
  can be retried by hand; an entry is never lost and never duplicated in Odoo.
- **Status chips** say where an entry stands: *Waiting to send*, *Not sent*
  (with the next retry time and a Retry button), *Draft* (in Odoo, not yet
  confirmed) and the confirmed states read back from Odoo.
- **The sign-in is the gate** (there is no PIN any more). If a phone's session
  has ended, a save brings the sign-in screen; the typed entry waits in the
  sheet and is sent once after sign-in, never twice.
- **The expense date is the Maldives date it was saved**, not the date it
  reached Odoo.
- The description Odoo shows is exactly what staff typed, or the category when
  they typed nothing.

The confirmed and pending spend shown here is read from
`https://raw.githubusercontent.com/yuki-uthman/ike-data/main/data/expenses.json`,
like the other panes. Only saving, listing the day's entries, the category list
and retry go to the Worker.

### The Worker

`worker/` is a Cloudflare Worker (name `odoo`) with a D1 database (`ike-odoo`).
It serves the page (`public/`) and its own routes from one origin. It is the only
part of the ike repos that writes to Odoo, and it holds the Odoo key as a Worker
secret; the page never sees it. Every data route (`/expenses/*`, `/quotations/*`)
needs a signed-in session, and every request that is not a read must carry the
Worker's own origin. Two Cloudflare crons run every 15 minutes: on the quarter
hour one retries unsent entries and reads their status back from Odoo (that
read is read-only); seven minutes later the other builds the open quotations
(read-only too, see above).

The decisions behind all of this are in `docs/product/brief.md` and
`docs/product/architecture/brief.md`; `worker/migrations/` is the database
schema. `npm test` runs the acceptance tests in `tests/` against the page in a
real browser and the Worker under Miniflare — both served together, exactly as
Cloudflare does — with no call to the real Odoo. Deploying the Worker is a
manual step by the owner.

## Visibility

The app itself is behind the sign-in: without it, only the sign-in screen
loads, and every Worker data route answers 401. What the sign-in does not
protect:

- **The page's source** (`public/`) is served to anyone, signed in or not; it
  holds no secret.
- **The Sales and Expenses figures come from ike-data's public files** on
  raw.githubusercontent.com. The sign-in hides them behind the screen but does
  not make them secret: anyone who knows the file URLs can read them, and the
  ike-data history keeps old ones, until that repo is made private. The
  Quotations pane reads the Worker, behind the sign-in, but ike-data still
  publishes the same quotations in its public `quotations.json`.
- ike-pos and ike-po have no sign-in, by design (staff use them on PCs).
