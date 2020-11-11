/* istanbul ignore file */
import axios from "axios"
import { fireEvent, render, wait } from "@testing-library/react"
import { constructGet } from "../../tests/api"
import React from "react"
import { MemoryRouter, Route, Redirect, Switch } from "react-router-dom"
import AdminDashboard from "./admin"

jest.mock("axios")
const mockedAxios = axios as jest.Mocked<typeof axios>

mockedAxios.get.mockImplementation(constructGet())

export function renderAdminPage(
  token?: string | null,
  path?: "users" | "all-projects"
) {
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
  const admin = renderAdminPage()
  await wait(() =>
    expect(admin.getByTestId("users-admin-widget")).toBeInTheDocument()
  )
  fireEvent.click(admin.getByText("All projects"))
  await wait(() => {
    expect(admin.getByTestId("projects-admin-widget")).toBeInTheDocument()
  })
})
