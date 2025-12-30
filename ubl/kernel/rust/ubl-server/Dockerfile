# UBL Server Dockerfile
# Multi-stage build for minimal image

# Stage 1: Build (using Alpine for musl-compatible binary)
FROM rust:1.75-alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig

WORKDIR /build

# Copy workspace
COPY Cargo.toml Cargo.lock ./
COPY ubl-kernel ./ubl-kernel
COPY ubl-atom ./ubl-atom
COPY ubl-link ./ubl-link
COPY ubl-ledger ./ubl-ledger
COPY ubl-membrane ./ubl-membrane
COPY ubl-pact ./ubl-pact
COPY ubl-policy-vm ./ubl-policy-vm
COPY ubl-runner-core ./ubl-runner-core
COPY ubl-server ./ubl-server

# Build release
RUN cargo build --release --package ubl-server

# Stage 2: Runtime
FROM alpine:3.19

RUN apk add --no-cache ca-certificates curl

WORKDIR /app

# Copy binary
COPY --from=builder /build/target/release/ubl-server /app/ubl-server

# Copy migrations if needed (commented out - migrations directory not found)
# COPY ubl-server/migrations /app/migrations

EXPOSE 8080

ENV HOST=0.0.0.0
ENV PORT=8080

CMD ["/app/ubl-server"]




