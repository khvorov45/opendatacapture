/* istanbul ignore file */
import axios from "axios"
import {
  fireEvent,
  render,
  wait,
  waitForDomChange,
  within,
} from "@testing-library/react"
import httpStatusCodes from "http-status-codes"
import toProperCase from "../../lib/to-proper-case"
import React from "react"
import { MemoryRouter, Route, Redirect, Switch } from "react-router-dom"
import ProjectPage from "./project"

jest.mock("axios")
const mockedAxios = axios as jest.Mocked<typeof axios>

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

test("data links", async () => {
  // This is to address multiple tabs highlighting as selected
  // https://github.com/khvorov45/opendatacapture/issues/86

  const mockedTables = ["tables", "subject", "subject-extra"]

  mockedAxios.get.mockResolvedValueOnce({
    status: httpStatusCodes.OK,
    data: mockedTables,
  })
  let data = renderProjectPage("123", "data")
  await waitForDomChange()

  const projectPageLinks = data.getByTestId("project-page-links")
  const tableDataLinks = data.getByTestId("table-data-links")

  // Check that the expected links are there
  mockedTables.map((t) =>
    expect(
      within(tableDataLinks).getByText(toProperCase(t))
    ).toBeInTheDocument()
  )

  // Check that only the expect links (Data and TableName) are active
  function expectActive(tableName: string) {
    expect(
      within(projectPageLinks).getByText("Tables").parentElement
    ).not.toHaveClass("active")
    expect(
      within(projectPageLinks).getByText("Data").parentElement
    ).toHaveClass("active")
    expect(
      within(tableDataLinks).getByText(toProperCase(tableName)).parentElement
    ).toHaveClass("active")
    mockedTables
      .filter((t) => t !== tableName)
      .map((t) =>
        expect(
          within(tableDataLinks).getByText(toProperCase(t)).parentElement
        ).not.toHaveClass("active")
      )
  }

  // Default selection is the first table
  expectActive(mockedTables[0])

  // Select the other tables
  mockedTables.slice(1).map(async (t) => {
    fireEvent.click(within(tableDataLinks).getByText(toProperCase(t)))
    await wait(() => expectActive(t))
  })
})
