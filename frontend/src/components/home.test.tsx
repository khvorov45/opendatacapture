/* istanbul ignore file */
import React from "react"
import axios from "axios"
import httpStatusCodes from "http-status-codes"
import Home, { ProjectWidget } from "./home"
import { render, waitForDomChange, fireEvent } from "@testing-library/react"
import { MemoryRouter as Router, Route } from "react-router"
import { constructGet, constructPut, constructDelete } from "../tests/api"

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

function renderHome(token: string | null) {
  return render(
    <Router>
      <Home token={token} />
    </Router>
  )
}

function renderProjectWidget() {
  return render(
    <Router>
      <ProjectWidget token="123" />
    </Router>
  )
}

test("homepage with no token", () => {
  const { queryByTestId } = renderHome(null)
  expect(queryByTestId("project-widget")).not.toBeInTheDocument()
})

test("new project form - no projects", async () => {
  mockedAxios.get.mockImplementation(
    constructGet({
      getUserProjects: async () => ({ status: httpStatusCodes.OK, data: [] }),
    })
  )
  const { getByTestId } = renderHome("123")
  await waitForDomChange()
  expect(getByTestId("homepage")).toBeInTheDocument()
  expect(getByTestId("project-widget")).toBeInTheDocument()
  expect(getByTestId("project-create-form")).not.toHaveClass("hidden")
})

test("new project form - some projects", async () => {
  const { getByTestId } = renderHome("123")
  await waitForDomChange()
  expect(getByTestId("homepage")).toBeInTheDocument()
  expect(getByTestId("project-widget")).toBeInTheDocument()
  expect(getByTestId("project-create-form")).toHaveClass("hidden")
})

test("new project form - open/close", async () => {
  const { getByTestId } = renderProjectWidget()
  await waitForDomChange()
  const form = getByTestId("project-create-form")
  const openFormButton = getByTestId("project-create-button")
  expect(form).toHaveClass("hidden")
  fireEvent.click(openFormButton)
  expect(form).not.toHaveClass("hidden")
  fireEvent.click(openFormButton)
  expect(form).toHaveClass("hidden")
})

test("project widget - create project", async () => {
  mockedAxios.put.mockResolvedValueOnce({ status: httpStatusCodes.NO_CONTENT })
  mockedAxios.get
    .mockResolvedValueOnce({
      status: httpStatusCodes.OK,
      data: [],
    })
    .mockResolvedValueOnce({
      status: httpStatusCodes.OK,
      data: [
        { user: 1, name: "newproject", created: new Date().toISOString() },
      ],
    })
  const { getByTestId, getByText } = renderProjectWidget()
  await waitForDomChange()
  expect(getByTestId("project-create-form")).not.toHaveClass("hidden")
  // Fill in the form
  fireEvent.change(getByTestId("project-name-field") as HTMLInputElement, {
    target: { value: "newproject" },
  })
  fireEvent.click(getByTestId("create-project-submit"))
  await waitForDomChange()
  expect(getByText("newproject")).toBeInTheDocument()
  // Create form should hide
  expect(getByTestId("project-create-form")).toHaveClass("hidden")
})

test("project widget - remove projects", async () => {
  mockedAxios.delete.mockResolvedValue({ status: httpStatusCodes.NO_CONTENT })
  mockedAxios.get
    .mockResolvedValueOnce({
      status: httpStatusCodes.OK,
      data: [{ user: 1, name: "2", created: new Date().toISOString() }],
    })
    .mockResolvedValueOnce({
      status: httpStatusCodes.OK,
      data: [],
    })
  const { getByTestId, queryByTestId } = renderProjectWidget()
  await waitForDomChange()
  expect(getByTestId("project-entry-2")).toBeInTheDocument()
  // Create form should be hidden
  expect(getByTestId("project-create-form")).toHaveClass("hidden")
  fireEvent.click(getByTestId("project-remove-2"))
  await waitForDomChange()
  expect(queryByTestId("project-entry-2")).not.toBeInTheDocument()
  // Create form should appear
  expect(getByTestId("project-create-form")).not.toHaveClass("hidden")
})

test("project widget - routing", async () => {
  mockedAxios.get.mockResolvedValueOnce({
    status: httpStatusCodes.OK,
    data: [
      { user: 1, name: "some-project", created: new Date().toISOString() },
    ],
  })
  const { getByText } = render(
    <Router>
      <Route exact to="/">
        <ProjectWidget token="123" />
      </Route>
      <Route exact to="/project/some-project">
        <div>Page for some-project</div>
      </Route>
    </Router>
  )
  await waitForDomChange()
  fireEvent.click(getByText("some-project"))
  expect(getByText("Page for some-project")).toBeInTheDocument()
})

test("project widget - fail to get projects", async () => {
  mockedAxios.get
    .mockRejectedValueOnce(Error("failed to get projects"))
    .mockResolvedValueOnce({
      status: httpStatusCodes.OK,
      data: [],
    })
  const { getByTestId } = render(<ProjectWidget token="123" />)
  await waitForDomChange()
  expect(getByTestId("project-control-error")).toHaveTextContent(
    "failed to get projects"
  )
  fireEvent.click(getByTestId("project-refresh-button"))
  await waitForDomChange()
  expect(getByTestId("project-control-error")).toHaveTextContent("")
})

test("project widget - project already exists", async () => {
  mockedAxios.get.mockResolvedValueOnce({
    status: httpStatusCodes.OK,
    data: [],
  })
  mockedAxios.put
    .mockRejectedValueOnce(Error("ProjectAlreadyExists"))
    .mockRejectedValueOnce(Error("some other error"))
    .mockResolvedValueOnce({ status: httpStatusCodes.NO_CONTENT })
  const { getByTestId } = renderProjectWidget()
  await waitForDomChange()
  fireEvent.change(getByTestId("project-name-field") as HTMLInputElement, {
    target: { value: "newproject" },
  })
  fireEvent.click(getByTestId("create-project-submit"))
  await waitForDomChange()
  expect(getByTestId("create-project-error")).toHaveTextContent(
    "Name 'newproject' already in use"
  )
  fireEvent.click(getByTestId("create-project-submit"))
  await waitForDomChange()
  expect(getByTestId("create-project-error")).toHaveTextContent(
    "some other error"
  )
  fireEvent.click(getByTestId("create-project-submit"))
  await waitForDomChange()
  expect(getByTestId("create-project-error")).toHaveTextContent("")
})

test("project widget - fail to delete project", async () => {
  mockedAxios.delete.mockRejectedValueOnce(Error("some delete project error"))
  mockedAxios.get.mockResolvedValueOnce({
    status: httpStatusCodes.OK,
    data: [{ user: 1, name: "prj", created: new Date().toISOString() }],
  })
  const { getByTestId } = renderProjectWidget()
  await waitForDomChange()
  expect(getByTestId("project-entry-buttons-error-prj")).toHaveTextContent("")
  fireEvent.click(getByTestId("project-remove-prj"))
  await waitForDomChange()
  expect(getByTestId("project-entry-buttons-error-prj")).toHaveTextContent(
    "some delete project error"
  )
})
