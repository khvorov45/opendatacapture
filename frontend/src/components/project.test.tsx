/* istanbul ignore file */
import React from "react"
import httpStatusCodes from "http-status-codes"
import {
  fireEvent,
  render,
  waitForDomChange,
  within,
} from "@testing-library/react"
import { BrowserRouter, Redirect, Route, Switch } from "react-router-dom"

import ProjectPage from "./project"

import axios from "axios"
import { ColMeta, TableMeta } from "../lib/project"
jest.mock("axios")
const mockedAxios = axios as jest.Mocked<typeof axios>

function renderProjectPage() {
  return render(
    <BrowserRouter>
      <Switch>
        <Route exact path="/">
          <Redirect to="/project/some-project" />
        </Route>
        <Route path="/project/:name">
          <ProjectPage token="123" />
        </Route>
      </Switch>
    </BrowserRouter>
  )
}

function performSelectAction(selectElement: HTMLElement, value: string) {
  fireEvent.mouseDown(within(selectElement).getByRole("button"))
  // Find the material ui popover
  // Note that it will hang around for some reason, even after the click
  // I'm guessing it's to mess with me
  const popovers = document.querySelectorAll<HTMLElement>(
    '[role="presentation"]'
  )
  if (popovers.length == 0) {
    throw Error("no popover from material ui")
  }
  // The last popover is the one we want
  fireEvent.click(within(popovers[popovers.length - 1]).getByText(value))
}

function performCheckboxClick(checkboxElement: HTMLElement) {
  const input = checkboxElement.querySelector('input[type="checkbox"]')
  if (!input) {
    throw Error("no checkbox input")
  }
  fireEvent.click(input)
}

function fillColumnEntry(columnEntry: HTMLElement, column: ColMeta) {
  const select = within(columnEntry)
  // Name
  fireEvent.change(select.getByTestId(`new-column-name-field`), {
    target: { value: column.name },
  })
  // Type
  performSelectAction(
    select.getByTestId(`new-column-type-select`),
    column.postgres_type
  )
  // Checkboxes
  if (column.primary_key) {
    performCheckboxClick(select.getByTestId("primary-key"))
  }
  if (column.not_null) {
    performCheckboxClick(select.getByTestId("not-null"))
  }
  if (column.unique) {
    performCheckboxClick(select.getByTestId("unique"))
  }
  if (column.foreign_key) {
    // Checkbox
    performCheckboxClick(select.getByTestId("foreing-key"))
    // Table
    performSelectAction(
      select.getByTestId(`foreign-table-select`),
      column.foreign_key.table
    )
    // Column
    performSelectAction(
      select.getByTestId(`foreign-column-select`),
      column.foreign_key.column
    )
  }
}

function fillTableForm(form: HTMLElement, table: TableMeta) {
  const select = within(form)
  // Fill in table name
  fireEvent.change(select.getByTestId("new-table-name-field"), {
    target: { value: table.name },
  })
  // Create the appropriate number of column entries
  for (let i = 1; i < table.cols.length; i++) {
    fireEvent.click(select.getByTestId("add-column"))
  }
  // Fill column entries
  table.cols.map((c, i) => {
    const colEntry = select.getByTestId(`new-column-entry-${i}`)
    fillColumnEntry(colEntry, c)
  })
}

test("table panel functionality - no initial tables", async () => {
  // List of tables
  mockedAxios.get.mockResolvedValue({ status: httpStatusCodes.OK, data: [] })

  let { getByTestId, getByText, queryByTestId } = renderProjectPage()
  await waitForDomChange()

  // Sidebar links
  expect(getByText("Tables")).toBeInTheDocument()

  // Open and close the new table form
  const newTableForm = getByTestId("new-table-form")
  expect(newTableForm).not.toHaveClass("nodisplay")
  fireEvent.click(getByTestId("create-table-button"))
  expect(newTableForm).toHaveClass("nodisplay")
  fireEvent.click(getByTestId("create-table-button"))
  expect(newTableForm).not.toHaveClass("nodisplay")

  // Submit button should be disabled
  const tableSubmit = getByTestId("submit-table-button")
  expect(tableSubmit).toBeDisabled()

  // Create a table
  const table1: TableMeta = {
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
    ],
  }
  fillTableForm(newTableForm, table1)

  // Submit button should now be enabled
  expect(tableSubmit).not.toBeDisabled()

  // Create table response
  let createTables = mockedAxios.put.mockImplementation(
    async (url, data, config) => {
      return { status: httpStatusCodes.NO_CONTENT }
    }
  )

  // Refresh tables response
  mockedAxios.get.mockResolvedValue({
    status: httpStatusCodes.OK,
    data: [table1],
  })

  // There should be no table cards
  expect(queryByTestId(`table-card-${table1.name}`)).not.toBeInTheDocument()

  // Submit table
  fireEvent.click(tableSubmit)
  await waitForDomChange()
  expect(createTables).toHaveBeenCalledWith(
    expect.anything(),
    table1,
    expect.anything()
  )

  // Now there should be a table card
  expect(getByTestId(`table-card-${table1.name}`)).toBeInTheDocument()

  // The table form should still be visible
  expect(newTableForm).not.toHaveClass("nodisplay")
  // And empty
  expect(within(newTableForm).getByTestId("new-table-name-field")).toHaveValue(
    ""
  )
})
