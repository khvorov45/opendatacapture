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

All request bodies and responses are in the `json` format.

|             Path              |  Type  |                Body                 |          Headers          |
| :---------------------------: | :----: | :---------------------------------: | :-----------------------: |
| `authenticate/email-password` | `POST` | `{email: String, password: String}` |                           |
|            `users`            | `GET`  |                                     | `Authorization: id:token` |
