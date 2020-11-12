/* istanbul ignore file */
import React from "react"
import { render } from "@testing-library/react"
import { Redirect, Route, Switch, MemoryRouter } from "react-router-dom"
import ProjectPage from "../components/project/project"
import { TableData, TableMeta } from "../lib/api/project"
import { Access, EmailPassword, User } from "../lib/api/auth"

export function renderProjectPage(
  token?: string | null,
  path?: "tables" | "data"
) {
  let tok: string | null = "123"
  if (token !== undefined) {
    tok = token
  }
  return render(
    <MemoryRouter
      initialEntries={[path ? `/project/some-project/${path}` : "/"]}
    >
      <Switch>
        <Route exact path="/">
          <Redirect to="/project/some-project" />
        </Route>
        <Route path="/project/:name">
          <ProjectPage token={tok} />
        </Route>
      </Switch>
    </MemoryRouter>
  )
}

export const defaultAdmin: User = {
  id: 1,
  email: "admin@example.com",
  access: Access.Admin,
}

export const newUser: EmailPassword = {
  email: "user@example.com",
  password: "user",
}

/* NOTE on postgres and primary keys:

When a column is marked as PRIMARY_KEY, the UNIQUE constraint is always absent
and the NOT_NULL "constraint" is always present regardless of what
SQL you write.

So all of the following will produce the same table:

CREATE TABLE some_table (id INT PRIMARY KEY)
CREATE TABLE some_table (id INT PRIMARY KEY NOT_NULL)
CREATE TABLE some_table (id INT PRIMARY KEY UNIQUE)
CREATE TABLE some_table (id INT PRIMARY KEY NOT_NULL UNIQUE)
*/

export const table1: TableMeta = {
  name: "newtable",
  cols: [
    {
      name: "id",
      postgres_type: "integer",
      primary_key: true,
      not_null: true,
      unique: false,
      foreign_key: null,
    },
    {
      name: "email",
      postgres_type: "text",
      primary_key: false,
      not_null: true,
      unique: true,
      foreign_key: null,
    },
    {
      name: "height",
      postgres_type: "real",
      primary_key: false,
      not_null: false,
      unique: false,
      foreign_key: null,
    },
    {
      name: "weight",
      postgres_type: "real",
      primary_key: false,
      not_null: false,
      unique: false,
      foreign_key: null,
    },
    {
      name: "male",
      postgres_type: "boolean",
      primary_key: false,
      not_null: false,
      unique: false,
      foreign_key: null,
    },
    {
      name: "dob",
      postgres_type: "timestamp with time zone",
      primary_key: false,
      not_null: false,
      unique: false,
      foreign_key: null,
    },
  ],
}

/* NOTE on dates:

Received dates are only decoded before presentation
Sent dates are decoded on input (JSON encoding sends them as strings though)
*/

export const table1data: TableData = [
  {
    id: 1,
    email: "e1@example.com",
    height: 170.5,
    weight: 60,
    male: false,
    dob: "2020-01-01T00:00:00.000Z",
  },
  {
    id: 2,
    email: "e2@example.com",
    height: 180,
    weight: 70.0,
    male: true,
    dob: "2020-01-01T00:00:00.000Z",
  },
]

// Compound primary key
export const table2: TableMeta = {
  name: "newtable2",
  cols: [
    {
      name: "id",
      postgres_type: "integer",
      primary_key: true,
      not_null: true,
      unique: false,
      foreign_key: { table: table1.name, column: table1.cols[0].name },
    },
    {
      name: "timepoint",
      postgres_type: "text",
      primary_key: true,
      not_null: true,
      unique: false,
      foreign_key: null,
    },
  ],
}

export const table2data = [
  { id: 1, timepoint: 1 },
  { id: 1, timepoint: 2 },
]

// No primary key
export const table3: TableMeta = {
  name: "newtable3",
  cols: [
    {
      name: "id",
      postgres_type: "integer",
      primary_key: false,
      not_null: false,
      unique: true,
      foreign_key: null,
    },
  ],
}

// Has the same name as one of its columns
export const tableTitre: TableMeta = {
  name: "titre",
  cols: [
    {
      name: "titre",
      postgres_type: "integer",
      primary_key: false,
      not_null: false,
      unique: false,
      foreign_key: null,
    },
  ],
}

export const tableTitreData: TableData = [{ titre: 20 }, { titre: 40 }]
