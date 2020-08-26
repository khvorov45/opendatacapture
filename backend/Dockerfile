FROM rust:latest

WORKDIR /usr/src/opendatacapture
COPY Cargo.toml .
COPY src ./src

RUN cargo build --release

FROM ubuntu:latest

COPY --from=0 /usr/src/opendatacapture/target/release/opendatacapture .

RUN apt-get update && apt-get install -y libssl-dev

ENV RUST_LOG info
ENV DATABASE_URL postgres://odcapi:odcapi@localhost:5432/odcadmin
ENV ODC_API_PORT 4321
ENV ODC_API_PREFIX ""
CMD ./opendatacapture
