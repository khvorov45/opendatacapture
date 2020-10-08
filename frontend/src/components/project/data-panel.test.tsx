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
    expect.anything(),
    [table1data[0]],
    expect.anything()
  )
  // Check data
  Object.entries(table1data[0]).map(([key, val]) => {
    expect(dataPanel.getByText(val.toString())).toBeInTheDocument()
  })
})
