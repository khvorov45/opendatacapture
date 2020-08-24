FROM rust:latest

WORKDIR /usr/src/opendatacapture
COPY . .

RUN cargo build --release

EXPOSE 4321

CMD ["cargo", "run", "--release", "--", "--dbhost", "postgres"]
