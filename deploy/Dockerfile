FROM rust:1.94-alpine3.22 AS builder

WORKDIR /workspace
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY deploy/vendor /vendor

ENV RUSTFLAGS="-C strip=symbols"
RUN cargo \
    --config 'source.crates-io.replace-with="vendored-sources"' \
    --config 'source.vendored-sources.directory="/vendor"' \
    build --offline --locked --release -p auth-server -p migration

FROM scratch

WORKDIR /app

COPY --from=builder /workspace/target/release/auth-server /usr/local/bin/auth-server
COPY --from=builder /workspace/target/release/migration /usr/local/bin/migration
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

EXPOSE 19878

CMD ["/usr/local/bin/auth-server"]
