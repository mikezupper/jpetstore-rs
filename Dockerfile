# The endgame: a statically linked musl binary in an empty image. The
# builder is Alpine (musl-native), build-base supplies the C compiler the
# bundled SQLite needs, and the runtime stage is FROM scratch — no shell,
# no package manager, no libc, nothing to patch or exploit. The image IS
# the binary, plus an empty /data directory and a numeric user.
FROM rust:1.95-alpine AS builder
RUN apk add --no-cache build-base
WORKDIR /app
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release \
    && mkdir -p /data && chown 10001:10001 /data

FROM scratch
COPY --from=builder /app/target/release/jpetstore-rs /jpetstore-rs
COPY --from=builder --chown=10001:10001 /data /data
USER 10001:10001
ENV DATABASE_URL=sqlite:/data/jpetstore.db
ENV BIND_ADDR=0.0.0.0:8081
EXPOSE 8081
VOLUME /data
ENTRYPOINT ["/jpetstore-rs"]
