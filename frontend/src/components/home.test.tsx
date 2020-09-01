/* istanbul ignore file */
import React from "react"
import axios from "axios"
import Home from "./home"
import { render, waitForDomChange } from "@testing-library/react"

jest.mock("axios")
const mockedAxios = axios as jest.Mocked<typeof axios>

function renderHome(token: string | null) {
  return render(<Home token={token} />)
}

test("homepage with no token", () => {
  const { queryByTestId } = renderHome(null)
  expect(queryByTestId("project-widget")).not.toBeInTheDocument()
})

test("homepage with token and no projects", async () => {
  // Get projects
  mockedAxios.get.mockResolvedValueOnce({ data: [] })
  const { getByTestId } = renderHome("123")
  await waitForDomChange()
  expect(getByTestId("homepage")).toBeInTheDocument()
  expect(getByTestId("project-widget")).toBeInTheDocument()
  // Check that the new project from is visible
  expect(getByTestId("project-create-form")).not.toHaveClass(
    "makeStyles-hidden-8"
  )
})

test("homepage with token and some projects", async () => {
  // Get projects
  mockedAxios.get.mockResolvedValueOnce({
    data: [{ user: 1, name: 2, created: new Date() }],
  })
  const { getByTestId } = renderHome("123")
  await waitForDomChange()
  expect(getByTestId("homepage")).toBeInTheDocument()
  expect(getByTestId("project-widget")).toBeInTheDocument()
  // Check that the new project form is hidden
  expect(getByTestId("project-create-form")).toHaveClass("makeStyles-hidden-17")
})
