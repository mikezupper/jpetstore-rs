# Port plan

The decisions governing this port, agreed 2026-08-11 before any code was
written. The course lessons follow this document; if the two ever disagree,
fix whichever one is wrong rather than letting them drift.

## What this is

A feature-parity port of [JPetStore 6](https://github.com/mybatis/jpetstore-6)
to Rust, built lesson by lesson for the free course
[Port a Classic Java App to Rust](https://mikezupper.com/courses/java-to-rust-jpetstore/).
Feature parity means: catalog browsing (category → product → item, with
inventory), search, session cart with quantity editing, accounts (register,
edit, sign in/out), checkout (shipping/billing → confirm → submit), and order
history/detail.

## Fidelity policy: faithful, with three deliberate fixes

Schema table names, URLs, seed data, and behavior stay recognizably
JPetStore. Exactly three modernizations, each flagged in its lesson with a
"what the original did, and why we won't" callout:

1. **Passwords are hashed with argon2.** The original stores them in
   plaintext in `SIGNON`.
2. **Card numbers are never persisted.** The original saves the full card
   number in `ORDERS`; we keep the checkout form shape but drop the storage,
   and the lesson explains what real systems do instead.
3. **Order IDs come from autoincrement.** The original's hand-rolled
   `SEQUENCE` table was a workaround for its era; SQLite doesn't need it.

Everything else ports as-is — including quirks that are merely dated rather
than harmful (favorite category, MyList, the banner option).

## Stack and dependency budget

Axum, Tokio, SQLx (SQLite), Askama, tower-sessions, argon2, serde, thiserror.
Roughly eight direct dependencies; lesson 12 holds that list up against the
original's Maven dependency tree as part of the final scorecard (dependency
count, binary size, memory).

## Code layout

One binary crate with clear modules (`domain/`, `db/`, `web/`, plus Askama
templates) — mirrors the original's package layout and keeps every lesson
navigable. A lesson aside covers when a Cargo workspace would earn its keep.

Data access is **plain async functions** in `db/` modules. No repository
traits until lesson 12, where testing supplies the pressure that justifies
extracting one. Abstraction is taught as a response to pressure, not a
ritual.

Money is integer cents (`i64`), never floats.

## Lesson outline

Each lesson ends with a running app, gets a git tag (`lesson-01` …), and
closes with one optional "now you" challenge. No solution branches; the tags
are the canonical checkpoints.

| # | Lesson | End state |
|---|---|---|
| 1 | The app you already know: JPetStore 6 tour, port plan, toolchain | original running via Docker; hello-world Axum server |
| 2 | Skeleton and error strategy | styled placeholder page + real error page |
| 3 | Schema and seed data as SQLx migrations | seeded SQLite file, app connects |
| 4 | Modeling the catalog: newtypes, parse-don't-validate | unit-tested catalog queries |
| 5 | Catalog pages with Askama | whole store browsable |
| 6 | Search | header search works |
| 7 | The cart: sessions, line items, integer cents | full cart CRUD with totals |
| 8 | Accounts: registration, argon2, auth extractor | sign-in works, cart survives it |
| 9 | Checkout forms (and the card-number lesson) | forms through to confirmation |
| 10 | Placing the order: one transaction | order placed, inventory decremented |
| 11 | Order history, order detail, account page | feature parity |
| 12 | Testing + final scorecard | integration & property-based tests; the wrap-up numbers |

## Out of scope (free course)

Docker/deployment, an API + client-side UI (CSR/SSR/SSG), observability, and
CI are reserved for paid follow-up courses. The free course ends at
`cargo run` serving the finished app.
