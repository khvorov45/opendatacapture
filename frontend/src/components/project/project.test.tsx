/* istanbul ignore file */
import React from "react"
import axios from "axios"
import {
  fireEvent,
  render,
  wait,
  waitForDomChange,
} from "@testing-library/react"
import { Redirect, Route, Switch, MemoryRouter } from "react-router-dom"
import ProjectPage from "./project"
import { TableMeta } from "../../lib/api/project"

// Need to mock so that API calls don't actually happen
jest.mock("axios")

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

test("project page with null token", async () => {
  // Normally the null (or wrong or too old)
  // token will fail to be verified in App which should
  // redirect us to login
  let { getByTestId } = renderProjectPage(null)
  // The top bar should be there
  expect(getByTestId("project-page-links")).toBeInTheDocument()
  // The main section should be absent
  expect(document.getElementsByTagName("main")).toBeEmpty()
})

test("links", async () => {
  let home = renderProjectPage()
  await waitForDomChange()
  expect(home.getByTestId("table-panel")).toBeInTheDocument()
  expect(home.getByText("Tables").parentElement).toHaveClass("active")
  fireEvent.click(home.getByText("Data"))
  await wait(() => {
    expect(home.getByText("Tables").parentElement).not.toHaveClass("active")
    expect(home.getByText("Data").parentElement).toHaveClass("active")
    expect(home.getByTestId("data-panel")).toBeInTheDocument()
  })
  fireEvent.click(home.getByText("Tables"))
  await waitForDomChange()
  expect(home.getByTestId("table-panel")).toBeInTheDocument()
})

test("render on table page", async () => {
  let tables = renderProjectPage("123", "tables")
  await waitForDomChange()
  expect(tables.getByTestId("table-panel")).toBeInTheDocument()
})

test("render on data page", async () => {
  let data = renderProjectPage("123", "data")
  await wait(() => expect(data.getByTestId("data-panel")).toBeInTheDocument())
})
