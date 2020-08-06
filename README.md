# opendatacapture

![Backend CI](https://github.com/khvorov45/opendatacapture/workflows/Backend%20CI/badge.svg?branch=master)
![Frontend CI](https://github.com/khvorov45/opendatacapture/workflows/Frontend%20CI/badge.svg?branch=master)
[![codecov](https://codecov.io/gh/khvorov45/opendatacapture/branch/master/graph/badge.svg)](https://codecov.io/gh/khvorov45/opendatacapture)

An open-source tool for capturing data in a relational database.

# Backend

Backend is written in
[Rust](https://www.rust-lang.org/)
with [tokio](https://github.com/tokio-rs/tokio) as the runtime,
[tokio-postgres](https://github.com/sfackler/rust-postgres)
as the [PostgreSQL](https://www.postgresql.org/) interface
and [warp](https://github.com/seanmonstar/warp) as the server framework.

# Frontend

Frontend is written in
[Typescript](https://www.typescriptlang.org/)
using the [React](https://reactjs.org/) framework
with [Material-UI](https://material-ui.com/) UI components.
Bootstrapped with
[Create React App](https://github.com/facebook/create-react-app).
