# Office Server Dockerfile
# Multi-stage build for minimal image

# Stage 1: Build
FROM rust:1.75-alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig

WORKDIR /build

# Copy the entire Office project
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build release
RUN cargo build --release

# Stage 2: Runtime
FROM alpine:3.19

RUN apk add --no-cache ca-certificates curl

WORKDIR /app

# Copy binary
COPY --from=builder /build/target/release/office /app/office

# Use 8081 to match docker-compose expectations
EXPOSE 8081

ENV HOST=0.0.0.0
ENV PORT=8081

CMD ["/app/office"]
