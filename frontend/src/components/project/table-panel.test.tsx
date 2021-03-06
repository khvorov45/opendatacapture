/* istanbul ignore file */
import httpStatusCodes from "http-status-codes"
import { fireEvent, render, within, waitFor } from "@testing-library/react"
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
let getreq: any
let putreq: any
let deletereq: any
beforeEach(() => {
  getreq = mockedAxios.get.mockImplementation(constructGet())
  putreq = mockedAxios.put.mockImplementation(constructPut())
  deletereq = mockedAxios.delete.mockImplementation(constructDelete())
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
  if (popovers.length === 0) {
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
  table.cols.forEach((c, i) => {
    const colEntry = select.getByTestId(`new-column-entry-${i}`)
    fillColumnEntry(colEntry, c)
  })
}

test("new table form - no initial tables", async () => {
  mockedAxios.get.mockImplementation(
    constructGet({
      getAllMeta: async () => ({ status: httpStatusCodes.OK, data: [] }),
    })
  )
  const tablePanel = renderTablePanel()
  await waitFor(() => {
    const newTableForm = tablePanel.getByTestId("new-table-form")
    expect(newTableForm).not.toHaveClass("nodisplay")
  })
})

test("new table form - some initial tables", async () => {
  const tablePanel = renderTablePanel()
  await waitFor(() => {
    const newTableForm = tablePanel.getByTestId("new-table-form")
    expect(newTableForm).toHaveClass("nodisplay")
  })
})

test("new table form - open/close", async () => {
  const tablePanel = renderTablePanel()
  await waitFor(() => {
    expect(tablePanel.getByTestId("new-table-form")).toBeInTheDocument()
  })
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
  await waitFor(() => {
    expect(tablePanel.getByTestId("new-table-form")).toBeInTheDocument()
  })
  const newTableForm = tablePanel.getByTestId("new-table-form")
  fillTableForm(newTableForm, table1)
  const tableSubmit = tablePanel.getByTestId("submit-table-button")
  fireEvent.click(tableSubmit)
  await waitFor(() => {
    expect(putreq).toHaveBeenCalledWith(
      expect.anything(),
      table1,
      expect.anything()
    )
  })
})

test("new table form - viability checks", async () => {
  let tablePanel = renderTablePanel()
  await waitFor(() => {
    expect(tablePanel.getByTestId("submit-table-button")).toBeInTheDocument()
  })

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
  await waitFor(() => {
    expect(getByTestId("create-table-button")).toBeInTheDocument()
  })

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
  await waitFor(() => {
    expect(putreq).toHaveBeenCalledWith(
      expect.anything(),
      newTable,
      expect.anything()
    )
  })
})

test("new table form - column removal", async () => {
  let { getByTestId } = renderTablePanel()
  await waitFor(() => {
    expect(getByTestId("new-table-form")).toBeInTheDocument()
  })

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
  await waitFor(() => {
    expect(tablePanel.getByTestId("refresh-tables-button")).toBeInTheDocument()
  })
  fireEvent.click(tablePanel.getByTestId("refresh-tables-button"))
  await waitFor(() => {
    expect(getreq).toHaveBeenLastCalledWith(
      `${API_ROOT}/project/${testProjectName}/get/meta`,
      expect.anything()
    )
  })
})

test("table cards presence", async () => {
  const tablePanel = renderTablePanel()
  const tableNames = (await defaultGet.getAllTableNames()).data
  await waitFor(() => {
    tableNames.forEach((t: string) => {
      expect(tablePanel.getByTestId(`table-card-${t}`)).toBeInTheDocument()
    })
  })
})

test("table card - remove table", async () => {
  const tablePanel = renderTablePanel()
  const table1Name = (await defaultGet.getAllTableNames()).data[0]
  await waitFor(() => {
    expect(
      tablePanel.getByTestId(`table-card-${table1Name}`)
    ).toBeInTheDocument()
  })

  const table1Card = tablePanel.getByTestId(`table-card-${table1Name}`)
  const deleteButton = within(table1Card).getByTestId("delete-table-button")
  fireEvent.click(deleteButton)
  await waitFor(() => {
    expect(deletereq).toHaveBeenLastCalledWith(
      `${API_ROOT}/project/${testProjectName}/remove/table/${table1Name}`,
      expect.anything()
    )
  })
})

test("table card - set editable", async () => {
  const firstTableName = (await defaultGet.getAllTableNames()).data[0]
  let { getByTestId } = renderTablePanel()
  await waitFor(() => {
    expect(getByTestId(`table-card-${firstTableName}`)).toBeInTheDocument()
  })
  // Table card should be disabled
  const card = getByTestId(`table-card-${firstTableName}`)
  const tableNameField = within(card).getByTestId("table-card-name-field")
  expect(tableNameField).toBeDisabled()
  // Enable editing
  fireEvent.click(within(card).getByTestId("enable-edit"))
  expect(tableNameField).not.toBeDisabled()
  // Disable editing
  fireEvent.click(within(card).getByTestId("enable-edit"))
  expect(tableNameField).toBeDisabled()
})

test("error - project refresh", async () => {
  mockedAxios.get.mockImplementation(
    constructGet({
      getAllMeta: async () => {
        throw Error("some refresh error")
      },
    })
  )
  let { getByTestId, getByText } = renderTablePanel()
  await waitFor(() => {
    expect(getByText("some refresh error")).toBeInTheDocument()
  })
  // Error should go away on successful refresh
  mockedAxios.get.mockImplementation(constructGet())
  fireEvent.click(getByTestId("refresh-tables-button"))
  await waitFor(() => {
    expect(getByTestId("refresh-tables-error")).toHaveTextContent("")
  })
})

test("error - table delete", async () => {
  mockedAxios.delete.mockImplementation(
    constructDelete({
      removeTable: async () => {
        throw Error("some delete error")
      },
    })
  )
  const firstTableName = (await defaultGet.getAllTableNames()).data[0]
  let { getByTestId, getByText } = renderTablePanel()
  await waitFor(() => {
    expect(getByTestId(`table-card-${firstTableName}`)).toBeInTheDocument()
  })
  let table1card = getByTestId(`table-card-${firstTableName}`)
  fireEvent.click(within(table1card).getByTestId("delete-table-button"))
  await waitFor(() => {
    expect(getByText("some delete error")).toBeInTheDocument()
  })

  // Error should go away on successful delete
  mockedAxios.delete.mockImplementation(constructDelete())
  fireEvent.click(within(table1card).getByTestId("delete-table-button"))
  await waitFor(() => {
    expect(
      within(table1card).getByTestId("delete-table-error")
    ).toHaveTextContent("")
  })
})

test("error - submit table", async () => {
  mockedAxios.put.mockImplementation(
    constructPut({
      createTable: async () => {
        throw Error("some table submit error")
      },
    })
  )
  let { getByTestId, getByText } = renderTablePanel()
  await waitFor(() => {
    expect(getByTestId("table-submit-error")).toHaveTextContent("")
  })

  fillTableForm(getByTestId("new-table-form"), table1)
  fireEvent.click(getByTestId("submit-table-button"))
  await waitFor(() => {
    expect(getByText("some table submit error")).toBeInTheDocument()
  })

  // Should go away on successful submit
  mockedAxios.put.mockImplementation(constructPut())
  fireEvent.click(getByTestId("submit-table-button"))
  await waitFor(() => {
    expect(getByTestId("table-submit-error")).toHaveTextContent("")
  })
})
