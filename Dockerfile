FROM rustlang/rust:nightly-bookworm-slim AS build

# add COMMIT_SHA as compile time env var
ARG COMMIT_SHA
ENV COMMIT_SHA=${COMMIT_SHA:-development}

RUN apt update && apt install -y \
    ca-certificates \
    pkg-config \
    && rm /var/lib/apt/lists/* -rf

WORKDIR /app

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app

# get up to date TLS certs
COPY --from=build /etc/ssl/certs/ /etc/ssl/certs/

COPY --from=build /app/target/release/vulpark .

EXPOSE 8000

CMD ["./vulpark"]
