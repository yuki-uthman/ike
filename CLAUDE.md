# ike-sales — how work is delivered here

The shared rules for all ike repos are in `../CLAUDE.md` (what each repo is,
commit style, Odoo access). This file adds how a change is built, checked and
landed in this repo, including by an unattended `/run-parked` worker.

## Where work happens

- **Worktree:** `/Users/Yuki/Claude/ike-apps/worktrees/ike-<N>-<slug>`, where
  `<N>` is the issue number. It sits inside `ike-apps/`, so `../CLAUDE.md`
  still loads. Never work in the primary checkout.
- **Branch:** `feat/<N>-<slug>` (or `fix/…` for a bug), from `origin/main`.

## Checks before pushing

All three must pass:

1. `npm test` (needs `npm ci` once per worktree).
2. `npx wrangler@4.148.0 deploy --dry-run --config worker/wrangler.json`: the
   Worker bundles.
3. `/code-review` over the branch diff. Fix what it finds, or list what was left
   and why in the hand-off.

## How work lands

- Push the branch and open a pull request whose body says `Closes #<N>`. Then
  stop: the issue goes to `awaiting-device`.
- **Only the owner deploys the Worker and merges the PR.** Merging to `main`
  publishes GitHub Pages at once, so a merge before the Worker is deployed can
  break the live app.
- A worker never runs `wrangler deploy` (except `--dry-run`), `wrangler secret`,
  `wrangler d1 execute --remote` or `wrangler d1 migrations apply --remote`,
  and never pushes to `main`.

## Odoo

A worker never calls the live Odoo, not even to read. Tests answer Odoo from
the fakes in `tests/`. Anything that can only be checked against the real Odoo
goes into the PR's device-check steps for the owner.
