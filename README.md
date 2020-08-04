# opendatacapture

![Backend CI](https://github.com/khvorov45/opendatacapture/workflows/Backend%20CI/badge.svg?branch=master)

An open-source tool for capturing data in a relational database.

# Backend

Backend is written in
[Rust](https://www.rust-lang.org/)
with [tokio](https://github.com/tokio-rs/tokio) as the runtime,
[tokio-postgres](https://github.com/sfackler/rust-postgres)
as the [PostgreSQL](https://www.postgresql.org/) interface
and [warp](https://github.com/seanmonstar/warp) as the server framework.
