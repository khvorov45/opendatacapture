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
    return { status: httpStatusCodes.OK, data: table1data }
  }
})

test("data panel functionality", async () => {
  const dataPanel = renderProjectPage("123", "data")
  await waitForDomChange()
  // The first table should be auto-selected
  const firstLink = dataPanel.getByText(toProperCase(table1.name))
  expect(firstLink).toBeInTheDocument()
  expect(firstLink.parentElement).toHaveClass("active")
})
