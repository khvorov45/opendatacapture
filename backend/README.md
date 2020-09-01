# Intended use

- Install Postgres, create the user and database for this api to
  connect to.

      - Postgres connection details can be passed to the api via command-line
      arguments at launch. The defaults are:
          - `--dbhost`: localhost
          - `--dbport`: 5432
          - `--admindbname`: odcadmin
          - `--apiusername`: odcapi
          - `--apiuserpassword`: odcapi

- Launch the app, pass the port it's supposed to listen to (`--apiport`) with
  the default being 4321.

- The database this api connects to is used the as admin database to keep
  track of users. If it's empty, it will be initialised
  (by creating the appropriate table structure).
  If it's not empty, the api will assume the database is correctly structured
  and do nothing. Pass `--clean` option to reset it (remove all tables and
  re-create them).

- With an empty database (or if `--clean` is passed), one new admin user will
  be automatically created with email `admin@example.com` and password `admin`.
  Pass `--admin-email` and `--admin-password` to override these defaults.

# API

All request bodies and responses are in the `json` format. All paths can have
a prefix added to them by passing the `--prefix` option at launch
(e.g. pass `--prefix api` to turn `get/users` into `api/get/users` )

## GET health

Returns `true` if able to successfully create a database connection
and `false` otherwise.

## POST auth/session-token

Body: `{email: String, password: String}`

Returns the authentication token.

## GET get/user/by/token/{token}

Return the user who the token belongs to.

## GET get/users

Header: `Authorization: Bearer <token>`

Authorization level: Admin

Returns all users.

## PUT create/project/{name}

Header: `Authorization: Bearer <token>`

Authorization level: User

Creates the user's project with the given name. User identified by token.

## DELETE delete/project/{name}

Header: `Authorization: Bearer <token>`

Authorization level: User

Deletes the user's project with the given name. User identified by token.

## GET get/projects

Header: `Authorization: Bearer <token>`

Authorization level: User

Returns all projects for the user who the token belongs to.
