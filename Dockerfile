# Two stages: a builder with the whole Rust toolchain, and a runtime that
# gets exactly one file from it. SQLX_OFFLINE makes the query macros verify
# against the committed .sqlx cache — the same property that lets a fresh
# clone compile (lesson 4) lets a build container compile with no database.
FROM rust:1.95-slim AS builder
WORKDIR /app
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release

# The runtime stage: the binary is the whole application — templates,
# migrations, and the pet pictures are compiled in. The only thing that
# lives outside it is the SQLite file on /data.
FROM debian:bookworm-slim
RUN useradd -r -u 10001 jpetstore && mkdir -p /data && chown jpetstore:jpetstore /data
COPY --from=builder /app/target/release/jpetstore-rs /usr/local/bin/jpetstore-rs
USER jpetstore
ENV DATABASE_URL=sqlite:/data/jpetstore.db
ENV BIND_ADDR=0.0.0.0:8081
EXPOSE 8081
VOLUME /data
CMD ["jpetstore-rs"]
