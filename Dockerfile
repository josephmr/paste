FROM rust:latest AS builder

RUN update-ca-certificates

ENV USER=runner
ENV UID=1000

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

WORKDIR /app

RUN touch /app/empty_db

COPY . .

RUN cargo build --release

# Final Image
# FROM gcr.io/distroless/cc
FROM ubuntu:latest

COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /app
COPY --from=builder /app/target/release/paste .
COPY --from=builder /app/Rocket.toml .
COPY --from=builder /app/migrations .

# Copy empty db into volume
COPY --from=builder --chown=1000:1000 /app/empty_db /app/db/paste.db

USER runner:runner

ENV PORT=8000

CMD ["/app/paste"]