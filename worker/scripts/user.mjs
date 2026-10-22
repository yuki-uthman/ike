#!/usr/bin/env node
// The owner's account tool (D26). It never touches a database itself: it PRINTS
// one SQL statement, which the owner runs against the live D1 with
//
//   npx wrangler d1 execute ike-odoo --remote --command "<the printed SQL>"
//
// so nothing in this repository holds a credential, and there is no admin route
// on the Worker for anyone to find.
//
//   node worker/scripts/user.mjs add <name>      new account, or a new password
//                                                for an existing one; the
//                                                generated password is printed once
//   node worker/scripts/user.mjs signout <name>  ends every phone's session
//   node worker/scripts/user.mjs remove <name>   ends the sessions and deletes the
//                                                account
//
// A password is never chosen by a person (D23): it is five words drawn from the
// 7,776-word list beside this file, joined by '-', about 64 bits of entropy.

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { hashPassword, newSalt, PASSWORD_ROUNDS } from '../src/auth.mjs';

const HERE = path.dirname(fileURLToPath(import.meta.url));
// The EFF large wordlist (https://www.eff.org/dice), words only. The original
// file's sha256 is addd35536511597a02fa0a9ff1e5284677b8883b83e986e43f15a3db996b903e.
const WORDLIST = path.join(HERE, 'wordlist.txt');
const WORDS_PER_PASSWORD = 5;

const USERNAME = /^[a-z0-9][a-z0-9._-]{0,31}$/;

function quote(text) {
  return `'${String(text).replace(/'/g, "''")}'`;
}

export function loadWords(file = WORDLIST) {
  const words = fs.readFileSync(file, 'utf8').split('\n').map(w => w.trim()).filter(Boolean);
  if (words.length !== 7776 || new Set(words).size !== 7776) {
    throw new Error(`${file} must hold 7,776 distinct words; it holds ${words.length}`);
  }
  return words;
}

/** A uniform index below `n`, by rejection, so no word is likelier than another. */
function uniformIndex(n) {
  const limit = Math.floor(0x100000000 / n) * n;
  const draw = new Uint32Array(1);
  for (;;) {
    crypto.getRandomValues(draw);
    if (draw[0] < limit) return draw[0] % n;
  }
}

export function generatePassword(words = loadWords()) {
  return Array.from({ length: WORDS_PER_PASSWORD }, () => words[uniformIndex(words.length)])
    .join('-');
}

export function checkUsername(name) {
  if (typeof name !== 'string' || !USERNAME.test(name)) {
    throw new Error('a username is 1-32 lower-case letters, digits, ".", "_" or "-",'
      + ' starting with a letter or digit');
  }
  return name;
}

/**
 * The statement that creates `name` with `password`, or gives an existing `name`
 * that password (and re-enables it). Existing sessions are left alone: ending
 * them is `signout`, so a new password plus `signout` removes someone (D22).
 */
export async function addUserSql(name, password, { salt = newSalt(), rounds = PASSWORD_ROUNDS,
  now = new Date() } = {}) {
  checkUsername(name);
  const hash = await hashPassword(password, salt, rounds);
  return 'INSERT INTO "user" (username, password_hash, salt, rounds, created_at)'
    + ` VALUES (${quote(name)}, ${quote(hash)}, ${quote(salt)}, ${rounds},`
    + ` ${quote(now.toISOString())})`
    + ' ON CONFLICT(username) DO UPDATE SET password_hash = excluded.password_hash,'
    + ' salt = excluded.salt, rounds = excluded.rounds, disabled_at = NULL;';
}

export function signoutSql(name) {
  checkUsername(name);
  return `DELETE FROM session WHERE user_id IN (SELECT id FROM "user" WHERE username = ${quote(name)});`;
}

export function removeSql(name) {
  return `${signoutSql(name)} DELETE FROM "user" WHERE username = ${quote(name)};`;
}

async function main(argv) {
  const [command, name] = argv;
  const usage = 'usage: node worker/scripts/user.mjs add|signout|remove <name>';
  if (!name || !['add', 'signout', 'remove'].includes(command)) {
    console.error(usage);
    return 2;
  }
  try {
    checkUsername(name);
  } catch (err) {
    console.error(err.message);
    return 2;
  }
  if (command === 'add') {
    const password = generatePassword();
    console.log(`Password for ${name} (shown once; write it down now):`);
    console.log(`  ${password}`);
    console.log('');
    console.log('Run this SQL with: npx wrangler d1 execute ike-odoo --remote --command "<SQL>"');
    console.log(await addUserSql(name, password));
    return 0;
  }
  console.log('Run this SQL with: npx wrangler d1 execute ike-odoo --remote --command "<SQL>"');
  console.log(command === 'signout' ? signoutSql(name) : removeSql(name));
  return 0;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).then(code => { process.exitCode = code; });
}
