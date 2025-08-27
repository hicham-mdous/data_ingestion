FROM rustlang/rust:nightly-alpine as builder

RUN apk add --no-cache musl-dev openssl-dev pkgconfig

WORKDIR /app
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:latest

RUN apk add --no-cache ca-certificates

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/data_ingestion /usr/local/bin/data_ingestion

CMD ["data_ingestion"]