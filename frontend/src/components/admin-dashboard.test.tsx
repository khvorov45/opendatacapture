/* istanbul ignore file */
import axios from "axios"
import {
  fireEvent,
  render,
  wait,
  waitForDomChange,
  within,
} from "@testing-library/react"
import httpStatusCodes from "http-status-codes"
import React from "react"
import { MemoryRouter, Route, Redirect, Switch } from "react-router-dom"
import AdminDashboard from "./admin-dashboard"

jest.mock("axios")
const mockedAxios = axios as jest.Mocked<typeof axios>

mockedAxios.get.mockImplementation(async (url) => {
  return { status: httpStatusCodes.OK, data: [] }
})

function renderAdmin(token?: string, path?: string) {
  let tok: string | null = "123"
  if (token !== undefined) {
    tok = token
  }
  return render(
    <MemoryRouter initialEntries={[path ? `/admin/${path}` : "/"]}>
      <Switch>
        <Route exact path="/">
          <Redirect to="/admin" />
        </Route>
        <Route path="/admin">
          <AdminDashboard token={tok} />
        </Route>
      </Switch>
    </MemoryRouter>
  )
}

test("navigation", async () => {
  const admin = renderAdmin()
  await wait(() =>
    expect(admin.getByTestId("users-admin-widget")).toBeInTheDocument()
  )
  fireEvent.click(admin.getByText("All projects"))
  await wait(() => {
    expect(admin.getByTestId("projects-admin-widget")).toBeInTheDocument()
  })
})
