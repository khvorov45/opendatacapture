FROM rust:latest

WORKDIR /usr/src/opendatacapture
COPY Cargo.toml .
COPY src ./src

RUN cargo build --release

FROM ubuntu:latest

COPY --from=0 /usr/src/opendatacapture/target/release/opendatacapture .

RUN apt-get update && apt-get install -y libssl-dev

ARG API_PORT=4321
EXPOSE ${API_PORT}

ENV RUST_LOG ${RUST_LOG}
ENV DATABASE_URL ${DATABASE_URL}
ENV ODC_API_PORT ${API_PORT}
ENV ODC_API_PREFIX ${API_PREFIX}
CMD ./opendatacapture
