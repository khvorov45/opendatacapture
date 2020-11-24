/* istanbul ignore file */
import httpStatusCodes from "http-status-codes"
import {
  fireEvent,
  render,
  waitForDomChange,
  within,
} from "@testing-library/react"
import axios from "axios"
import { TableMeta, ColMeta } from "../../lib/api/project"
import { API_ROOT } from "../../lib/config"
import { table1, table2, table3 } from "../../tests/data"
import React from "react"
import TablePanel from "./table-panel"
import {
  constructDelete,
  constructGet,
  constructPut,
  defaultGet,
} from "../../tests/api"

jest.mock("axios")
const mockedAxios = axios as jest.Mocked<typeof axios>
const getreq = mockedAxios.get.mockImplementation(constructGet())
const putreq = mockedAxios.put.mockImplementation(constructPut())
const deletereq = mockedAxios.delete.mockImplementation(constructDelete())
afterEach(() => {
  mockedAxios.get.mockImplementation(constructGet())
  mockedAxios.put.mockImplementation(constructPut())
  mockedAxios.delete.mockImplementation(constructDelete())
})

const testProjectName = "some-project"

export function renderTablePanel() {
  return render(<TablePanel token="123" projectName={testProjectName} />)
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
    performCheckboxClick(select.getByTestId("foreign-key"))
    // Table
    performSelectAction(
      select.getByTestId(`foreign-table-select`),
      column.foreign_key.table
    )
    // No need to select a column since there could only be one choice per
    // table which is selected automatically and the input is always disabled
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

// Thanks, Material UI, for easy-to-test components
function expectMuiSelectToBeEmpty(select: HTMLElement) {
  const textContent = within(select).getByRole("button").childNodes[0]
    .textContent
  // Was this really necessary or was it just to mess with me?
  expect(textContent).toEqual("\u200B")
}

function expectTableFormToBeEmpty(form: HTMLElement) {
  expect(within(form).getByTestId("new-table-name-field")).toHaveValue("")
  // The following should fail if there is more than one column entry
  expect(within(form).getByTestId("new-column-name-field")).toHaveValue("")
  expectMuiSelectToBeEmpty(within(form).getByTestId("new-column-type-select"))
  expect(within(form).getByTestId("primary-key")).not.toBeChecked()
  expect(within(form).getByTestId("not-null")).not.toBeChecked()
  expect(within(form).getByTestId("unique")).not.toBeChecked()
  expect(within(form).getByTestId("foreign-key")).not.toBeChecked()
}

test("new table form - no initial tables", async () => {
  mockedAxios.get.mockImplementation(
    constructGet({
      getAllMeta: async () => ({ status: httpStatusCodes.OK, data: [] }),
    })
  )
  const tablePanel = renderTablePanel()
  await waitForDomChange()
  const newTableForm = tablePanel.getByTestId("new-table-form")
  expect(newTableForm).not.toHaveClass("nodisplay")
})

test("new table form - some initial tables", async () => {
  const tablePanel = renderTablePanel()
  await waitForDomChange()
  const newTableForm = tablePanel.getByTestId("new-table-form")
  expect(newTableForm).toHaveClass("nodisplay")
})

test("new table form - open/close", async () => {
  const tablePanel = renderTablePanel()
  await waitForDomChange()
  const newTableForm = tablePanel.getByTestId("new-table-form")
  expect(newTableForm).toHaveClass("nodisplay")
  const createTableButton = tablePanel.getByTestId("create-table-button")
  fireEvent.click(createTableButton)
  expect(newTableForm).not.toHaveClass("nodisplay")
  fireEvent.click(createTableButton)
  expect(newTableForm).toHaveClass("nodisplay")
})

test("new table form - fill and submit", async () => {
  const tablePanel = renderTablePanel()
  await waitForDomChange()
  const newTableForm = tablePanel.getByTestId("new-table-form")
  fillTableForm(newTableForm, table1)
  const tableSubmit = tablePanel.getByTestId("submit-table-button")
  fireEvent.click(tableSubmit)
  await waitForDomChange()
  expect(putreq).toHaveBeenCalledWith(
    expect.anything(),
    table1,
    expect.anything()
  )
})

test("new table form - viability checks", async () => {
  let tablePanel = renderTablePanel()
  await waitForDomChange()

  const tableSubmit = tablePanel.getByTestId("submit-table-button")

  // Should be disabled since the form is empty
  expect(tableSubmit).toBeDisabled()

  // Enter table name
  fireEvent.change(tablePanel.getByTestId("new-table-name-field"), {
    target: { value: table1.name },
  })
  // Should be disabled since the one column doesn't have a name nor a type
  expect(tableSubmit).toBeDisabled()

  // Enter column name
  fireEvent.change(
    within(tablePanel.getByTestId("new-column-entry-0")).getByTestId(
      "new-column-name-field"
    ),
    {
      target: { value: table1.cols[0].name },
    }
  )
  // Should be disabled since the one column doesn't have a type
  expect(tableSubmit).toBeDisabled()

  // Enter type
  performSelectAction(
    within(tablePanel.getByTestId("new-column-entry-0")).getByTestId(
      "new-column-type-select"
    ),
    table1.cols[0].postgres_type
  )
  // Should not be disabled
  expect(tableSubmit).not.toBeDisabled()

  // Add a column
  fireEvent.click(tablePanel.getByTestId("add-column"))
  // Should be disabled since the second column is empty
  expect(tableSubmit).toBeDisabled()
})

test("new table form - FK behavior", async () => {
  let { getByTestId, getAllByRole } = renderTablePanel()
  await waitForDomChange()

  fireEvent.click(getByTestId("create-table-button"))
  const newTableForm = getByTestId("new-table-form")
  performCheckboxClick(within(newTableForm).getByTestId("foreign-key"))
  const foreignTable = within(newTableForm).getByTestId("foreign-table-select")
  const foreignColumn = within(newTableForm).getByTestId(
    "foreign-column-select"
  )
  // Foreign table should be auto-selected
  expect(foreignTable).toHaveTextContent(table1.name)
  // Foreign column selection should be disabled
  expect(within(foreignColumn).getByRole("button")).toHaveAttribute(
    "aria-disabled"
  )
  // Only the first table should be available
  fireEvent.mouseDown(within(foreignTable).getByRole("button"))
  const popovers = getAllByRole("presentation")
  const popover = popovers[popovers.length - 1]
  expect(within(popover).getByText(table1.name)).toBeInTheDocument()
  expect(within(popover).queryByText(table2.name)).not.toBeInTheDocument()
  expect(within(popover).queryByText(table3.name)).not.toBeInTheDocument()
  fireEvent.click(within(popover).getByText(table1.name))
  // Make the new table viable
  const newTable: TableMeta = {
    name: "fkTest",
    cols: [
      {
        name: "fkTestCol",
        postgres_type: "integer",
        not_null: false,
        unique: false,
        primary_key: false,
        foreign_key: null,
      },
    ],
  }
  fillTableForm(newTableForm, newTable)
  // The foreign key constraint should still be there
  expect(foreignTable).toHaveTextContent(table1.name)
  // Now remove the foreign key constraint
  performCheckboxClick(within(newTableForm).getByTestId("foreign-key"))

  // See that the expected table would be created
  fireEvent.click(getByTestId("submit-table-button"))
  await waitForDomChange()
  expect(putreq).toHaveBeenCalledWith(
    expect.anything(),
    newTable,
    expect.anything()
  )
})

test("new table form - column removal", async () => {
  let { getByTestId } = renderTablePanel()
  await waitForDomChange()

  const newTableForm = getByTestId("new-table-form")
  fillTableForm(newTableForm, table1)

  // Remove from the begining
  fireEvent.click(
    within(within(newTableForm).getByTestId("new-column-entry-0")).getByTestId(
      "remove-column"
    )
  )

  // Check that the second column shifted up
  expect(
    within(within(newTableForm).getByTestId("new-column-entry-0")).getByTestId(
      "new-column-name-field"
    )
  ).toHaveValue(table1.cols[1].name)

  // Remove from the middle
  fireEvent.click(
    within(within(newTableForm).getByTestId("new-column-entry-1")).getByTestId(
      "remove-column"
    )
  )
  // Check that the 4th column is now second
  expect(
    within(within(newTableForm).getByTestId("new-column-entry-1")).getByTestId(
      "new-column-name-field"
    )
  ).toHaveValue(table1.cols[3].name)

  // Remove from the bottom
  for (let i = table1.cols.length - 3; i >= 1; i--) {
    fireEvent.click(
      within(
        within(newTableForm).getByTestId(`new-column-entry-${i}`)
      ).getByTestId("remove-column")
    )
  }
  // There should be one left
  expect(
    within(newTableForm).queryByTestId("new-column-entry-1")
  ).not.toBeInTheDocument()
  expect(
    within(within(newTableForm).getByTestId("new-column-entry-0")).getByTestId(
      "new-column-name-field"
    )
  ).toHaveValue(table1.cols[1].name)

  // Remove the only column
  fireEvent.click(
    within(within(newTableForm).getByTestId("new-column-entry-0")).getByTestId(
      "remove-column"
    )
  )
  // Check that the new column form is still there but empty
  expect(
    within(within(newTableForm).getByTestId("new-column-entry-0")).getByTestId(
      "new-column-name-field"
    )
  ).toHaveValue("")
})

test("refresh button", async () => {
  const tablePanel = renderTablePanel()
  await waitForDomChange()
  fireEvent.click(tablePanel.getByTestId("refresh-tables-button"))
  await waitForDomChange()
  expect(getreq).toHaveBeenLastCalledWith(
    `${API_ROOT}/project/${testProjectName}/get/meta`,
    expect.anything()
  )
})

test("table cards presence", async () => {
  const tablePanel = renderTablePanel()
  await waitForDomChange()
  const tableNames = (await defaultGet.getAllTableNames()).data
  tableNames.map((t: string) => {
    expect(tablePanel.getByTestId(`table-card-${t}`)).toBeInTheDocument()
  })
})

test("table card - remove table", async () => {
  const tablePanel = renderTablePanel()
  await waitForDomChange()
  const table1Name = (await defaultGet.getAllTableNames()).data[0]
  const table1Card = tablePanel.getByTestId(`table-card-${table1Name}`)
  const deleteButton = within(table1Card).getByTestId("delete-table-button")
  fireEvent.click(deleteButton)
  await waitForDomChange()
  expect(deletereq).toHaveBeenLastCalledWith(
    `${API_ROOT}/project/${testProjectName}/remove/table/${table1Name}`,
    expect.anything()
  )
})

test("table panel functionality - no initial tables", async () => {
  // List of tables
  mockedAxios.get.mockResolvedValue({ status: httpStatusCodes.OK, data: [] })

  let { getByTestId, getByText, queryByTestId } = renderTablePanel()
  await waitForDomChange()

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
  expectTableFormToBeEmpty(newTableForm)

  // Create another table
  fillTableForm(newTableForm, table2)

  // Refresh tables response
  mockedAxios.get.mockResolvedValue({
    status: httpStatusCodes.OK,
    data: [table1, table2],
  })

  // Need to get again because it gets replaced with a spinner and then
  // re-rendered
  fireEvent.click(getByTestId("submit-table-button"))
  await waitForDomChange()
  expect(createTables).toHaveBeenCalledWith(
    expect.anything(),
    table2,
    expect.anything()
  )
  expectTableFormToBeEmpty(newTableForm)

  // There should be another table card
  expect(getByTestId(`table-card-${table1.name}`)).toBeInTheDocument()
  expect(getByTestId(`table-card-${table2.name}`)).toBeInTheDocument()

  // Delete a table
  const deleteTable = mockedAxios.delete.mockImplementation(
    async (url, config) => {
      return { status: httpStatusCodes.NO_CONTENT }
    }
  )
  mockedAxios.get.mockResolvedValue({
    status: httpStatusCodes.OK,
    data: [table1],
  })
  fireEvent.click(
    within(getByTestId(`table-card-${table2.name}`)).getByTestId(
      "delete-table-button"
    )
  )
  await waitForDomChange()
  expect(deleteTable).toHaveBeenCalledWith(
    `${API_ROOT}/project/some-project/remove/table/${table2.name}`,
    expect.anything()
  )
  expect(queryByTestId(`table-card-${table2.name}`)).not.toBeInTheDocument()
  expect(getByTestId(`table-card-${table1.name}`)).toBeInTheDocument()
}, 20000)

test("table panel - project refresh error", async () => {
  mockedAxios.get
    .mockRejectedValueOnce(Error("some refresh error"))
    .mockResolvedValueOnce({
      status: httpStatusCodes.OK,
      data: [],
    })
  let { getByTestId, getByText } = renderTablePanel()
  await waitForDomChange()
  expect(getByText("some refresh error")).toBeInTheDocument()
  fireEvent.click(getByTestId("refresh-tables-button"))
  await waitForDomChange()
  expect(getByTestId("refresh-tables-error")).toHaveTextContent("")
})

test("table panel - table delete error", async () => {
  mockedAxios.get
    // On load
    .mockResolvedValueOnce({
      status: httpStatusCodes.OK,
      data: [table1],
    })
    // On successful delete
    .mockResolvedValueOnce({
      status: httpStatusCodes.OK,
      data: [],
    })
  mockedAxios.delete
    .mockRejectedValueOnce(Error("some delete error"))
    .mockResolvedValueOnce({
      status: httpStatusCodes.NO_CONTENT,
    })
  let { getByTestId, getByText, queryByTestId } = renderTablePanel()
  await waitForDomChange()
  let table1card = getByTestId(`table-card-${table1.name}`)
  fireEvent.click(within(table1card).getByTestId("delete-table-button"))
  await waitForDomChange()
  expect(getByText("some delete error")).toBeInTheDocument()
  fireEvent.click(within(table1card).getByTestId("delete-table-button"))
  await waitForDomChange()
  expect(queryByTestId(`table-card-${table1.name}`)).not.toBeInTheDocument()
})

test("submit table error", async () => {
  mockedAxios.get
    .mockResolvedValueOnce({ status: httpStatusCodes.OK, data: [] })
    .mockResolvedValueOnce({ status: httpStatusCodes.OK, data: [table1] })
  mockedAxios.put
    .mockRejectedValueOnce(Error("some table submit error"))
    .mockResolvedValueOnce({ status: httpStatusCodes.NO_CONTENT })
  let { getByTestId } = renderTablePanel()
  await waitForDomChange()
  expect(getByTestId("table-submit-error")).toHaveTextContent("")
  fillTableForm(getByTestId("new-table-form"), table1)
  fireEvent.click(getByTestId("submit-table-button"))
  await waitForDomChange()
  expect(getByTestId("table-submit-error")).toHaveTextContent(
    "some table submit error"
  )
  fireEvent.click(getByTestId("submit-table-button"))
  await waitForDomChange()
  expect(getByTestId("table-submit-error")).toHaveTextContent("")
})

test("set table card to editable", async () => {
  mockedAxios.get.mockResolvedValue({
    status: httpStatusCodes.OK,
    data: [table1],
  })
  let { getByTestId } = renderTablePanel()
  await waitForDomChange()
  // Table card should be disabled
  const card = getByTestId(`table-card-${table1.name}`)
  expect(within(card).getByTestId("table-card-name-field")).toBeDisabled()
  // Enable editing
  fireEvent.click(within(card).getByTestId("enable-edit"))
  expect(within(card).getByTestId("table-card-name-field")).not.toBeDisabled()
  // Disable editing
  fireEvent.click(within(card).getByTestId("enable-edit"))
  expect(within(card).getByTestId("table-card-name-field")).toBeDisabled()
})
