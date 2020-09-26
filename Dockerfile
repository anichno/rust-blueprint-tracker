FROM rust as builder

RUN apt-get update && apt-get install -y cmake
COPY src/server /build
WORKDIR /build
RUN cargo build --release


FROM debian:buster-slim

COPY --from=builder /build/target/release/server /app/
COPY src/server/schema.rdf /app/
WORKDIR /app
ENTRYPOINT ["/app/server"]