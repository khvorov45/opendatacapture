/* istanbul ignore file */
import httpStatusCodes from "http-status-codes"
import {
  fireEvent,
  render,
  wait,
  waitForDomChange,
  within,
} from "@testing-library/react"
import axios from "axios"
import {
  table1,
  table2,
  table3,
  table1data,
  table2data,
} from "../../tests/data"
import toProperCase from "../../lib/to-proper-case"
import { getAllTableNames, TableRow } from "../../lib/api/project"
import { API_ROOT } from "../../lib/config"
import { decodeUserTable } from "../../lib/api/io-validation"
import React from "react"
import { MemoryRouter, Route, Redirect, Switch } from "react-router-dom"
import DataPanel from "./data-panel"
import { constructGet, defaultGet } from "../../tests/api"

jest.mock("axios")
const mockedAxios = axios as jest.Mocked<typeof axios>

const mockProjectName = "some-project"

export function renderDataPanel() {
  return render(
    <MemoryRouter>
      <Switch>
        <Route exact path="/">
          <Redirect to={`/project/${mockProjectName}/data`} />
        </Route>
        <Route path="/project/:name/data">
          <DataPanel token="123" projectName={mockProjectName} />
        </Route>
      </Switch>
    </MemoryRouter>
  )
}

const getreq = mockedAxios.get.mockImplementation(constructGet())

const putreq = mockedAxios.put.mockImplementation(async (url, data) => ({
  status: httpStatusCodes.NO_CONTENT,
}))

const deletereq = mockedAxios.delete.mockImplementation(async (url) => ({
  status: httpStatusCodes.NO_CONTENT,
}))

function selectFieldByLabel(region: HTMLElement, fieldName: string) {
  const field = within(region)
    .getByText(fieldName) // label
    .parentElement?.querySelector("input[type='text']") // sibling input
  if (field) {
    return field
  } else {
    throw Error("field not found")
  }
}

function fillNewRow(newRow: HTMLElement, data: TableRow) {
  Object.entries(data).map(([key, val]) => {
    fireEvent.change(selectFieldByLabel(newRow, key), {
      target: { value: val },
    })
  })
}

test("links", async () => {
  const dataPanel = renderDataPanel()
  await waitForDomChange()
  const allLinksText = (
    await defaultGet.getAllTableNames()
  ).data.map((n: string) => toProperCase(n))
  allLinksText.map((l: string) =>
    expect(dataPanel.getByText(l)).toBeInTheDocument()
  )
})

test("new row form - open/close", async () => {
  const dataPanel = renderDataPanel()
  await waitForDomChange()
  const headers = dataPanel.getByTestId("header-row")
  const inputRow = dataPanel.getByTestId("input-row")
  const newRowToggle = within(headers).getByTestId("new-row-toggle")
  // Initially not visible because some fake data is fetched by default
  expect(inputRow).toHaveClass("nodisplay")
  // Start clicking on the show/hide button
  fireEvent.click(newRowToggle)
  expect(inputRow).not.toHaveClass("nodisplay")
  fireEvent.click(newRowToggle)
  expect(inputRow).toHaveClass("nodisplay")
})

test("refresh", async () => {
  const dataPanel = renderDataPanel()
  await waitForDomChange()
  const refreshButton = dataPanel.getByTestId("refresh-table-button")
  fireEvent.click(refreshButton)
  await waitForDomChange()
  const firstTable = (await defaultGet.getAllTableNames()).data[0]
  expect(getreq).toHaveBeenLastCalledWith(
    `${API_ROOT}/project/${mockProjectName}/get/table/${firstTable}/data`,
    expect.anything()
  )
  expect(getreq).toHaveBeenNthCalledWith(
    getreq.mock.calls.length - 1,
    `${API_ROOT}/project/${mockProjectName}/get/table/${firstTable}/meta`,
    expect.anything()
  )
})

test("delete", async () => {
  const dataPanel = renderDataPanel()
  await waitForDomChange()
  const deleteButton = dataPanel.getByTestId("delete-all-table-data-button")
  fireEvent.click(deleteButton)
  await waitForDomChange()
  const firstTable = (await defaultGet.getAllTableNames()).data[0]
  expect(deletereq).toHaveBeenLastCalledWith(
    `${API_ROOT}/project/${mockProjectName}/remove/${firstTable}/all`,
    expect.anything()
  )
})

test("new row form - no data", async () => {
  // Should open automatically when page loads and there is no data
  mockedAxios.get.mockImplementation(
    constructGet({
      getTableData: async () => ({ status: httpStatusCodes.OK, data: [] }),
    })
  )
  // Render
  const dataPanel = renderDataPanel()
  await waitForDomChange()
  const inputRow = dataPanel.getByTestId("input-row")
  expect(inputRow).not.toHaveClass("nodisplay")

  // Now hide
  const headers = dataPanel.getByTestId("header-row")
  const newRowToggle = within(headers).getByTestId("new-row-toggle")
  fireEvent.click(newRowToggle)
  expect(inputRow).toHaveClass("nodisplay")

  // Now refresh data - should show up automatically
  const refreshButton = dataPanel.getByTestId("refresh-table-button")
  fireEvent.click(refreshButton)
  await waitForDomChange()
  expect(inputRow).not.toHaveClass("nodisplay")

  mockedAxios.get.mockImplementation(constructGet())
})

test("insert row", async () => {
  const dataPanel = renderDataPanel()
  await waitForDomChange()
  const inputRow = dataPanel.getByTestId("input-row")
  mockedAxios.get.mockResolvedValueOnce({
    status: httpStatusCodes.OK,
    data: [table1data[0]],
  })
  fillNewRow(inputRow, table1data[0])
  fireEvent.click(within(inputRow).getByTestId("submit-row-button"))
  await waitForDomChange()
  expect(putreq).toHaveBeenCalledWith(
    `${API_ROOT}/project/some-project/insert/${table1.name}`,
    decodeUserTable(table1, [table1data[0]]),
    expect.anything()
  )
})

test("fail to get a list of tables", async () => {
  mockedAxios.get.mockImplementation(
    constructGet({
      getAllTableNames: async () => {
        throw Error("get tables error")
      },
    })
  )
  const dataPanel = renderDataPanel()
  await waitForDomChange()
  expect(dataPanel.getByText("get tables error")).toBeInTheDocument()
  mockedAxios.get.mockImplementation(constructGet())
})

test("fail to fetch data and meta", async () => {
  mockedAxios.get.mockImplementation(
    constructGet({
      getTableData: async () => {
        throw Error("fetch error")
      },
      getTableMeta: async () => {
        throw Error("fetch error")
      },
    })
  )
  const dataPanel = renderDataPanel()
  await waitForDomChange()
  expect(dataPanel.getByText("fetch errorfetch error")).toBeInTheDocument()
  mockedAxios.get.mockImplementation(constructGet())
})

test("fail to fetch data/meta after a successful fetch", async () => {
  const dataPanel = renderDataPanel()
  await waitForDomChange()
  mockedAxios.get.mockImplementation(
    constructGet({
      getTableData: async () => {
        throw Error("fetch error")
      },
      getTableMeta: async () => {
        throw Error("fetch error")
      },
    })
  )
  fireEvent.click(dataPanel.getByTestId("refresh-table-button"))
  await waitForDomChange()
  expect(dataPanel.getByText("fetch errorfetch error")).toBeInTheDocument()
  mockedAxios.get.mockImplementation(constructGet())
})

test("fail to submit", async () => {
  const dataPanel = renderDataPanel()
  await waitForDomChange()
  mockedAxios.put.mockRejectedValueOnce(Error("submit error"))
  fireEvent.click(dataPanel.getByTestId("submit-row-button"))
  await waitForDomChange()
  expect(dataPanel.getByText("submit error")).toBeInTheDocument()
})

test("fail to delete", async () => {
  const dataPanel = renderDataPanel()
  await waitForDomChange()
  mockedAxios.delete.mockRejectedValueOnce(Error("delete error"))
  fireEvent.click(dataPanel.getByTestId("delete-all-table-data-button"))
  await waitForDomChange()
  expect(dataPanel.getByText("delete error")).toBeInTheDocument()
})

test("no tables", async () => {
  mockedAxios.get.mockImplementation(
    constructGet({
      getAllTableNames: async () => ({ status: httpStatusCodes.OK, data: [] }),
    })
  )
  const dataPanel = renderDataPanel()
  await waitForDomChange()
  expect(dataPanel.getByText("No tables found")).toBeInTheDocument()
  mockedAxios.get.mockImplementation(constructGet())
})

test("fill a new field entry and then remove what's been filled", async () => {
  const dataPanel = renderDataPanel()
  await waitForDomChange()
  const inputRow = dataPanel.getByTestId("input-row")
  fillNewRow(inputRow, table1data[0])
  const newRecord: TableRow = { ...table1data[0] }
  delete newRecord[Object.keys(table1data)[0]]
  fireEvent.change(
    selectFieldByLabel(inputRow, Object.keys(table1data[0])[0]),
    { target: { value: "" } }
  )
  fireEvent.click(within(inputRow).getByTestId("submit-row-button"))
  await waitForDomChange()
  expect(putreq).toHaveBeenCalledWith(
    `${API_ROOT}/project/some-project/insert/${table1.name}`,
    decodeUserTable(table1, [newRecord]),
    expect.anything()
  )
})

test("attempt to put a string into a number field", async () => {
  const dataPanel = renderDataPanel()
  await waitForDomChange()
  const inputRow = dataPanel.getByTestId("input-row")
  fillNewRow(inputRow, table1data[0])
  const inputToMod = selectFieldByLabel(inputRow, "id")
  fireEvent.change(inputToMod, { target: { value: "a" } })
  expect(inputToMod).toBeInvalid()
})

test("meta/data mismatch", async () => {
  mockedAxios.get.mockImplementation(
    constructGet({
      getTableMeta: async () => ({ status: httpStatusCodes.OK, data: table1 }),
      getTableData: async () => ({
        status: httpStatusCodes.OK,
        data: table2data,
      }),
    })
  )
  const dataPanel = renderDataPanel()
  await waitForDomChange()
  expect(dataPanel.queryAllByTestId("data-row")).toBeEmpty()
  mockedAxios.get.mockImplementation(constructGet())
})
