FROM rust:1.71.1-alpine AS build

RUN apk update && apk add ca-certificates && rm -rf /var/cache/apk/*

WORKDIR /app

COPY . .

RUN cargo build --release

FROM alpine

WORKDIR /app

# get up to date TLS certs
COPY --from=build /etc/ssl/certs/ /etc/ssl/certs/

COPY --from=build /app/target/release/vulpark .

EXPOSE 8000

CMD ["./vulpark"]
