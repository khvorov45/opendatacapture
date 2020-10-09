/* istanbul ignore file */
import httpStatusCodes from "http-status-codes"
import { fireEvent, waitForDomChange, within } from "@testing-library/react"
import axios from "axios"
import {
  renderProjectPage,
  table1,
  table2,
  table3,
  table1data,
} from "../../tests/util"
import toProperCase from "../../lib/to-proper-case"
import { TableRow } from "../../lib/api/project"
import { API_ROOT } from "../../lib/config"

jest.mock("axios")
const mockedAxios = axios as jest.Mocked<typeof axios>

mockedAxios.get.mockImplementation(async (url) => {
  if (url.endsWith("/get/tablenames")) {
    return {
      status: httpStatusCodes.OK,
      data: [table1.name, table2.name, table3.name],
    }
  }
  if (url.endsWith(`/get/table/${table1.name}/meta`)) {
    return { status: httpStatusCodes.OK, data: table1 }
  }
  if (url.endsWith(`/get/table/${table1.name}/data`)) {
    return { status: httpStatusCodes.OK, data: [] }
  }
})

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

test("data panel functionality", async () => {
  const dataPanel = renderProjectPage("123", "data")
  await waitForDomChange()
  // The first table should be auto-selected
  const firstLink = dataPanel.getByText(toProperCase(table1.name))
  expect(firstLink).toBeInTheDocument()
  expect(firstLink.parentElement).toHaveClass("active")
  // Check headers
  const headers = dataPanel.getByTestId("header-row")
  table1.cols.map((c) =>
    expect(within(headers).getByText(c.name)).toBeInTheDocument()
  )
  // Check that the new row is displayed
  const inputRow = dataPanel.getByTestId("input-row")
  expect(inputRow).not.toHaveClass("nodisplay")
  // Add a row
  mockedAxios.get.mockResolvedValueOnce({
    status: httpStatusCodes.OK,
    data: [table1data[0]],
  })
  fillNewRow(inputRow, table1data[0])
  fireEvent.click(within(inputRow).getByTestId("submit-row-button"))
  await waitForDomChange()
  expect(putreq).toHaveBeenCalledWith(
    `${API_ROOT}/project/some-project/insert/${table1.name}`,
    [table1data[0]],
    expect.anything()
  )
  // Check data
  Object.entries(table1data[0]).map(([key, val]) => {
    expect(dataPanel.getByText(val.toString())).toBeInTheDocument()
  })
  // Close, open and close the new row form
  fireEvent.click(within(headers).getByTestId("new-row-toggle"))
  expect(inputRow).toHaveClass("nodisplay")
  fireEvent.click(within(headers).getByTestId("new-row-toggle"))
  expect(inputRow).not.toHaveClass("nodisplay")
  fireEvent.click(within(headers).getByTestId("new-row-toggle"))
  expect(inputRow).toHaveClass("nodisplay")
  // Delete all table data
  fireEvent.click(within(headers).getByTestId("delete-all-table-data-button"))
  await waitForDomChange()
  expect(deletereq).toHaveBeenCalledWith(
    `${API_ROOT}/project/some-project/remove/${table1.name}/all`,
    expect.anything()
  )
  // Check that the new row form is opened automatically
  expect(inputRow).not.toHaveClass("nodisplay")
})

test("fail to get a list of tables", async () => {
  mockedAxios.get.mockRejectedValueOnce(Error("get tables error"))
  const dataPanel = renderProjectPage("123", "data")
  await waitForDomChange()
  expect(dataPanel.getByText("get tables error")).toBeInTheDocument()
})

test("fail to fetch data and meta", async () => {
  mockedAxios.get
    // Table names
    .mockResolvedValueOnce({ status: httpStatusCodes.OK, data: ["table"] })
    // Meta/data, order is unknown
    .mockRejectedValueOnce(Error("fetch error"))
    .mockRejectedValueOnce(Error("fetch error"))
  const dataPanel = renderProjectPage("123", "data")
  await waitForDomChange()
  expect(dataPanel.getByText("fetch errorfetch error")).toBeInTheDocument()
})

test("fail to fetch data/meta after a successful fetch", async () => {
  const dataPanel = renderProjectPage("123", "data")
  await waitForDomChange()
  mockedAxios.get
    .mockResolvedValueOnce({ status: httpStatusCodes.OK, data: ["table"] })
    .mockRejectedValueOnce(Error("fetch error"))
    .mockRejectedValueOnce(Error("fetch error"))
  fireEvent.click(dataPanel.getByTestId("refresh-table-button"))
  await waitForDomChange()
  expect(dataPanel.getByText("fetch errorfetch error")).toBeInTheDocument()
})

test("fail to submit", async () => {
  const dataPanel = renderProjectPage("123", "data")
  await waitForDomChange()
  mockedAxios.put.mockRejectedValueOnce(Error("submit error"))
  fireEvent.click(dataPanel.getByTestId("submit-row-button"))
  await waitForDomChange()
  expect(dataPanel.getByText("submit error")).toBeInTheDocument()
})

test("fail to delete", async () => {
  const dataPanel = renderProjectPage("123", "data")
  await waitForDomChange()
  mockedAxios.delete.mockRejectedValueOnce(Error("delete error"))
  fireEvent.click(dataPanel.getByTestId("delete-all-table-data-button"))
  await waitForDomChange()
  expect(dataPanel.getByText("delete error")).toBeInTheDocument()
})

test("no tables", async () => {
  mockedAxios.get.mockResolvedValueOnce({
    status: httpStatusCodes.OK,
    data: [],
  })
  const dataPanel = renderProjectPage("123", "data")
  await waitForDomChange()
  expect(dataPanel.getByText("No tables found")).toBeInTheDocument()
})

test("fill a new field entry and then remove what's been filled", async () => {
  const dataPanel = renderProjectPage("123", "data")
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
    [newRecord],
    expect.anything()
  )
})
