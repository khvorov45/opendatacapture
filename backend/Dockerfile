FROM rust:latest

WORKDIR /usr/src/opendatacapture
COPY Cargo.toml .
COPY src ./src

RUN cargo build --release

FROM ubuntu:latest

COPY --from=0 /usr/src/opendatacapture/target/release/opendatacapture .

RUN apt-get update && apt-get install -y libssl-dev

ENTRYPOINT ["./opendatacapture"]
