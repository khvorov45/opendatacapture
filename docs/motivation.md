# Motivation for opendatacapture

## Current state of data capture

The way data is currently captured for research and public health needs
ranges from bad to worse:

- [Proprietary data capture systems](https://www.capterra.com.au/directory/30551/electronic-data-capture/software). Bad because you have to pay for them. No
  budget alloted for this - stuck with worse options.

- [Access](https://www.microsoft.com/en-au/microsoft-365/access).
  Bad because not web-based, so collaborators all need access to the same
  file system. It also locks its data behind all
  sorts of Microsoft Nonsense™©® making it difficult to directly incorporate
  into (say) an [R](https://www.r-project.org/)-based workflow.

- [REDCap](https://www.project-redcap.org/). Bad because the website front-end
  for it looks hideous, its UI is all over the place, and the data is stored as
  one big table-like structure that can't be queried in any meaningful way.

- Random documents (usually Excel files) scattered all over the place.
  Bad because it's random documents scattered all over the place.

## Aims

- To have a proper relational database
  (e.g. [PostgreSQL](https://www.postgresql.org/)) used for data storage.

- To have a REST API for manipulating the users and their individual databases.

- To have a website front-end that's not a nightmare to navigate.

- To be open-source and make it possible for everyone with a computer to run the
  whole app (if only locally).
  None of this ["schedule a demo"](https://www.dacimasoftware.com/contact)
  nonsense.

## Envisioned setup

1. Start the DBMS.
2. Create an empty database.
3. Create a user with database creation/deletion privileges.
4. Start the backend API service by having it connect to the empty database as
   the created user.
5. Host the website front-end. Supply it with the address of the backend.

## Envisioned use

In general, direct manipulation of the databases should not be
necessary.

### Managing users

Users and administrators can be added in a number of ways

- A stock sign-up page optionally available to everyone with access to the
  website.

- Admin panel available to administrators.

- External sign-up pages (e.g. on the organisation's website) that add users
  through the API.

### Managing databases

Users should be able to use the website to

- Create databases to use for projects.

- Create tables within the databases.

- Define foreign key relationships between tables.

- Fill tables directly.

- Create forms that fill different fields across different tables.

- Host created forms as part of the website.

- Provide (e.g., email) access links to non-users (e.g., participants) which
  pre-fill certain fields (e.g., participant ID).

- Create data views that join fields across tables.

- Export data from the tables and views.

## Why this is better

- Anyone (with a server that's visible on the internet) can host it
  on the Internet and anyone with a computer can host it locally.

- Anyone (who knows how) can extend it or modify it (open-source).

- DBMS allows for a flexible and robust way of defining multiple tables
  (and their relationships) to describe the data capturing process.

- API with a proper table structure would allow direct incorporation into
  workflows (no need to export manually every time).

- DBMS enforces data integrity (e.g., can't enter a duplicate ID).
