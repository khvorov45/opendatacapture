/* istanbul ignore file */
import axios from "axios"
import { fireEvent, wait, waitForDomChange } from "@testing-library/react"
import { renderProjectPage } from "../../tests/util"

// Need to mock so that API calls don't actually happen
jest.mock("axios")

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
