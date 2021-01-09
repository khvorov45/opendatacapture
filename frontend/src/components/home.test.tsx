/* istanbul ignore file */
import React from "react"
import axios from "axios"
import httpStatusCodes from "http-status-codes"
import Home, { ProjectWidget } from "./home"
import { render, fireEvent, waitFor } from "@testing-library/react"
import { MemoryRouter as Router, Route } from "react-router"
import {
  constructGet,
  constructPut,
  constructDelete,
  defaultGet,
} from "../tests/api"
import { API_ROOT } from "../lib/config"

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
  await waitFor(() => {
    expect(getByTestId("homepage")).toBeInTheDocument()
    expect(getByTestId("project-widget")).toBeInTheDocument()
    expect(getByTestId("project-create-form")).not.toHaveClass("hidden")
  })
})

test("new project form - some projects", async () => {
  const { getByTestId } = renderHome("123")
  await waitFor(() => {
    expect(getByTestId("homepage")).toBeInTheDocument()
    expect(getByTestId("project-widget")).toBeInTheDocument()
    expect(getByTestId("project-create-form")).toHaveClass("hidden")
  })
})

test("new project form - open/close", async () => {
  const { getByTestId } = renderProjectWidget()
  const form = getByTestId("project-create-form")
  const openFormButton = getByTestId("project-create-button")
  await waitFor(() => {
    expect(form).toHaveClass("hidden")
  })
  fireEvent.click(openFormButton)
  expect(form).not.toHaveClass("hidden")
  fireEvent.click(openFormButton)
  expect(form).toHaveClass("hidden")
})

test("refresh button", async () => {
  const { getByTestId } = renderProjectWidget()
  await waitFor(() => {
    expect(getByTestId("project-refresh-button")).toBeInTheDocument()
  })
  fireEvent.click(getByTestId("project-refresh-button"))
  await waitFor(() => {
    expect(getreq).toHaveBeenLastCalledWith(
      `${API_ROOT}/get/projects`,
      expect.anything()
    )
  })
})

test("create project", async () => {
  const { getByTestId } = renderProjectWidget()
  await waitFor(() => {
    expect(getByTestId("project-name-field")).toBeInTheDocument()
  })
  // Fill in the form
  fireEvent.change(getByTestId("project-name-field") as HTMLInputElement, {
    target: { value: "newproject" },
  })
  fireEvent.click(getByTestId("create-project-submit"))
  await waitFor(() => {
    expect(putreq).toHaveBeenLastCalledWith(
      `${API_ROOT}/create/project/newproject`,
      expect.anything(),
      expect.anything()
    )
  })
})

test("remove a project", async () => {
  const { getByTestId } = renderProjectWidget()
  const firstProjectName = (await defaultGet.getUserProjects()).data[0].name
  await waitFor(() => {
    expect(getByTestId(`project-entry-${firstProjectName}`)).toBeInTheDocument()
  })
  // Create form should be hidden
  fireEvent.click(getByTestId(`project-remove-${firstProjectName}`))
  await waitFor(() => {
    expect(deletereq).toHaveBeenCalledWith(
      `${API_ROOT}/delete/project/${firstProjectName}`,
      expect.anything()
    )
  })
})

test("project widget - routing", async () => {
  const firstProjectName = (await defaultGet.getUserProjects()).data[0].name
  const { getByText } = render(
    <Router>
      <Route exact to="/">
        <ProjectWidget token="123" />
      </Route>
      <Route exact to={`/project/${firstProjectName}`}>
        <div>Page for some project</div>
      </Route>
    </Router>
  )
  await waitFor(() => {
    expect(getByText(firstProjectName)).toBeInTheDocument()
  })
  fireEvent.click(getByText(firstProjectName))
  expect(getByText("Page for some project")).toBeInTheDocument()
})

test("error - fetch projects", async () => {
  mockedAxios.get.mockImplementation(
    constructGet({
      getUserProjects: async () => {
        throw Error("failed to get projects")
      },
    })
  )
  const { getByTestId, getByText } = renderProjectWidget()
  await waitFor(() => {
    expect(getByText("failed to get projects")).toBeInTheDocument()
  })
  const error = getByTestId("project-control-error")
  // Should go away on success
  mockedAxios.get.mockImplementation(constructGet())
  fireEvent.click(getByTestId("project-refresh-button"))
  await waitFor(() => {
    expect(error).toHaveTextContent("")
  })
})

test("error - project already exists", async () => {
  mockedAxios.put.mockImplementation(
    constructPut({
      createProject: async () => {
        throw Error("ProjectAlreadyExists")
      },
    })
  )
  const { getByTestId, getByText } = renderProjectWidget()
  await waitFor(() => {
    expect(getByTestId("project-name-field")).toBeInTheDocument()
  })
  fireEvent.change(getByTestId("project-name-field") as HTMLInputElement, {
    target: { value: "newproject" },
  })
  fireEvent.click(getByTestId("create-project-submit"))
  await waitFor(() => {
    expect(getByText("Name 'newproject' already in use")).toBeInTheDocument()
  })
  mockedAxios.put.mockImplementation(
    constructPut({
      createProject: async () => {
        throw Error("some other error")
      },
    })
  )
  fireEvent.click(getByTestId("create-project-submit"))
  await waitFor(() => {
    expect(getByText("some other error")).toBeInTheDocument()
  })
})

test("error - fail to delete project", async () => {
  const firstProjectName = (await defaultGet.getUserProjects()).data[0].name
  mockedAxios.delete.mockImplementation(
    constructDelete({
      deleteProject: async () => {
        throw Error("some delete project error")
      },
    })
  )
  const { getByTestId, getByText } = renderProjectWidget()
  await waitFor(() => {
    expect(
      getByTestId(`project-entry-buttons-error-${firstProjectName}`)
    ).toHaveTextContent("")
  })
  fireEvent.click(getByTestId(`project-remove-${firstProjectName}`))
  await waitFor(() => {
    expect(getByText("some delete project error")).toBeInTheDocument()
  })
})
