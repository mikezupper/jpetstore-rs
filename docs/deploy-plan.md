# Ship the Port — deploy-course plan

The production spec — now the production *record* — for paid course #1, **"Ship the Port: Deploy Rust to a
Server You Own"** (slug `ship-the-port`), scoped 2026-08-12. The
[port plan](port-plan.md)'s lesson authoring contract applies unchanged:
every new pattern shown once in full, a Build-it table with an objective
done-when, one challenge per lesson, answer keys as tag-to-tag diffs.

**Status: BUILT 2026-08-13.** All nine checkpoints tagged (`deploy-01` …
`deploy-09`, every one verified live: the 7.73 MB scratch image, real
registry pushes and pulls, the compose healthcheck via the self-probing
binary, and the full Litestream restore drill — which surfaced the
root-owned-restore-file crash-loop now taught in lesson 8). Lessons 01–09
are written, linted, gated behind learn.mikezupper.com, and absent from
the static build. The musl verification the plan required passed on the
first attempt.

## Premise and audience

The direct sequel to the free course, which ends at `cargo run` and
promises this. The student owns a working binary and has never had to be
their own ops team; nine lessons later the pet store is on a domain they
own, with TLS, tested backups, and a rollback plan. Everything taught here
is something this project's own platform deployment actually did.

Paid: **$5** (the ladder's entry rung; also unlocked by the $19
all-access). Lessons live in the blog repo under
`src/lessons/ship-the-port/`, served only through learn.mikezupper.com's
entitlement gate — the static site never renders them. The catalog entry
gets created when course production starts, *after* the current branches
merge (the `tasks/free-course` branch changed the course frontmatter
schema; don't author against the old one).

## Decisions

- **Code lives here**, continuing jpetstore-rs history with annotated tags
  `deploy-01` … `deploy-09` after `lesson-12`. Public code, paid prose:
  buyers pay for the guided path and the guarantee, not secrecy.
- **Ingress is Caddy** — automatic TLS from a three-line Caddyfile as one
  more compose service. nginx/certbot and Cloudflare Tunnel get a
  paragraph of "when you'd choose differently," not lessons.
- **Target is a bare VPS over SSH with plain docker compose.** Universal
  and transferable. Portainer appears as a short aside: the same compose
  file, operated through a UI — the author's real setup.
- **The image endgame is `FROM scratch`** via a static musl build,
  continuing the one-binary narrative into a one-binary *image*.

## Lesson outline

| # | Lesson | End state |
|---|---|---|
| 1 | Config crosses the boundary — env vars for bind address and database path | same binary serves dev and prod; defaults unchanged |
| 2 | The Dockerfile — multi-stage, toolchain confined to the builder | `docker run` serves the store locally |
| 3 | The endgame image — musl static build, `FROM scratch` | an image barely bigger than the binary |
| 4 | A registry and a build script — sha tags, `:latest`, login | pushed, pullable from anywhere |
| 5 | Compose — the service, a `/healthz`, the external volume | one file describes the deployment; data survives redeploys |
| 6 | The server — bare VPS to running stack over SSH | the store serves on a public IP |
| 7 | A domain and TLS — Caddy joins the stack | https with automatic certificates |
| 8 | Backups that are real — Litestream sidecar + a restore drill | database replicates off-box; a restore has been *performed* |
| 9 | Updates, rollbacks, and the scorecard | ship a change, roll back by tag; image/memory/cold-start numbers |

## Production notes (course-writing time)

- **Verify the musl build before writing lesson 3.** Bundled SQLite and
  pure-Rust argon2 should compile clean under `x86_64-unknown-linux-musl`;
  if reality disagrees, the lesson lands honestly on distroless instead —
  the narrative survives either way.
- **Lesson 5 adds a `/healthz` route** to the app (a few lines, mirroring
  the platform's) so the compose healthcheck has something honest to ask.
- Lesson 1's diff is the env-var change sketched and reverted during
  platform work (bind address + `DATABASE_URL`, defaults preserving course
  behavior) — small on purpose; the lesson is about *why*, not the code.
- Lesson 8 is the course's quiet star: "backups are only real once restore
  is tested" is this project's stated belief — the lesson makes the
  student live it before trusting a production database to lesson 9.
