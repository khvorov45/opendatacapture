/* istanbul ignore file */
import React from "react"
import { render } from "@testing-library/react"
import { Redirect, Route, Switch, MemoryRouter } from "react-router-dom"
import ProjectPage from "../components/project/project"
import { TableMeta } from "../lib/api/project"

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

export const table1: TableMeta = {
  name: "newtable",
  cols: [
    {
      name: "id",
      postgres_type: "integer",
      primary_key: true,
      not_null: false,
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
      postgres_type: "integer",
      primary_key: false,
      not_null: false,
      unique: false,
      foreign_key: null,
    },
    {
      name: "weight",
      postgres_type: "integer",
      primary_key: false,
      not_null: false,
      unique: false,
      foreign_key: null,
    },
  ],
}

export const table1data = [
  { id: 1, email: "e1@example.com", height: 170, weight: 60 },
]

// Compound primary key
export const table2: TableMeta = {
  name: "newtable2",
  cols: [
    {
      name: "id",
      postgres_type: "integer",
      primary_key: true,
      not_null: false,
      unique: false,
      foreign_key: { table: table1.name, column: table1.cols[0].name },
    },
    {
      name: "timepoint",
      postgres_type: "text",
      primary_key: true,
      not_null: false,
      unique: false,
      foreign_key: null,
    },
  ],
}

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
