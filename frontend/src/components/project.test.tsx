/* istanbul ignore file */
import React from "react"
import { fireEvent, render, waitForDomChange } from "@testing-library/react"
import { BrowserRouter, Redirect, Route, Switch } from "react-router-dom"

import ProjectPage from "./project"

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

test("basic functionality - no initial tables", async () => {
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
