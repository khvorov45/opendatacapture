/* istanbul ignore file */
import React from "react"
import httpStatusCodes from "http-status-codes"
import { fireEvent, render, waitForDomChange } from "@testing-library/react"
import { BrowserRouter, Redirect, Route, Switch } from "react-router-dom"

import ProjectPage from "./project"

import axios from "axios"
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

test("table panel functionality - no initial tables", async () => {
  // List of projects
  mockedAxios.get.mockResolvedValue({ status: httpStatusCodes.OK, data: [] })

  let { getByTestId, queryByTestId, getByText } = renderProjectPage()
  await waitForDomChange()

  // Sidebar links
  expect(getByText("Tables")).toBeInTheDocument()

  // Open and close the new table form
  expect(getByTestId("new-table-form")).not.toHaveClass("nodisplay")
  fireEvent.click(getByTestId("create-table-button"))
  expect(queryByTestId("new-table-form")).toHaveClass("nodisplay")
  fireEvent.click(getByTestId("create-table-button"))
  expect(getByTestId("new-table-form")).not.toHaveClass("nodisplay")
})
