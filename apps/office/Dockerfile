# Office Server Dockerfile
# Multi-stage build for minimal image

# Stage 1: Build
FROM rust:1.75-alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig

WORKDIR /build

# Copy Office crate
COPY office/Cargo.toml office/Cargo.lock ./office/
COPY office/src ./office/src

# Copy UBL dependencies
COPY ubl/kernel/rust/ubl-kernel ../ubl/kernel/rust/ubl-kernel
COPY ubl/kernel/rust/ubl-atom ../ubl/kernel/rust/ubl-atom

WORKDIR /build/office

# Build release
RUN cargo build --release

# Stage 2: Runtime
FROM alpine:3.19

RUN apk add --no-cache ca-certificates curl

WORKDIR /app

# Copy binary
COPY --from=builder /build/office/target/release/office /app/office

EXPOSE 8787

ENV HOST=0.0.0.0
ENV PORT=8787

CMD ["/app/office"]
