# Intended use

- Install Postgres, create the user and database for this api to
connect to.

    - Postgres connection details can be passed to the api via command-line
    arguments at launch. The defaults are:
        - dbhost: localhost
        - dbport: 5432
        - admindbname: odcadmin
        - apiusername: odcapi
        - apiuserpassword: odcapi

- Launch the app, pass the port it's supposed to listen to (--apiport) with
the default being 4321.

- The database this api connects to is used the as admin database to keep
track of users. If it's empty, it will be initialised
(by creating the appropriate table structure).
If its table structure is correct it won't be immediately modified.
If its table structure is incorrect,
the api will throw an error unless it was given the `--forcereset` option in
which case it will attempt to drop all tables in that database and then
initialise it.
