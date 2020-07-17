# Intended use

- Install Postgres, create the default user and database for this api to
connect to.

    - Postgres connection details can be passed to the api via command-line
    arguments at launch. The defaults are:
        - dbhost: localhost
        - dbport: 5432
        - dbname: odcdefault
        - dbuser: odcdefault
        - dbpassword: odcdefault

- Launch the app, pass the port it's supposed to listen to (--apiport) with
the default being 4321.
