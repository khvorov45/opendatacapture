/* istanbul ignore file */
import axios from "axios"
import {
  fireEvent,
  render,
  wait,
  waitForDomChange,
} from "@testing-library/react"
import httpStatusCodes from "http-status-codes"
import { defaultAdmin } from "../../tests/util"
import { constructGet } from "../../tests/api"
import { user1 } from "../../tests/data"
import React from "react"
import Users from "./users"

jest.mock("axios")
const mockedAxios = axios as jest.Mocked<typeof axios>

mockedAxios.get.mockImplementation(constructGet())

function renderUsers() {
  return render(<Users token="123" />)
}

test("new user form open/close", async () => {
  const users = renderUsers()
  await wait(() => {
    expect(users.getByTestId("users-admin-widget")).toBeInTheDocument()
  })
  const frm = users.getByTestId("new-user-form")
  expect(frm).toHaveClass("nodisplay")
  const btn = users.getByTestId("open-new-user-form-button")
  fireEvent.click(btn)
  expect(frm).not.toHaveClass("nodisplay")
  fireEvent.click(btn)
  expect(frm).toHaveClass("nodisplay")
})

test("refresh users", async () => {
  const users = renderUsers()
  await wait(() => {
    expect(users.getByTestId("users-admin-widget")).toBeInTheDocument()
  })
  // Only admin should be in the table
  expect(users.getByText(defaultAdmin.email)).toBeInTheDocument()
  expect(users.queryByText(user1.email)).not.toBeInTheDocument()

  // Refresh
  mockedAxios.get.mockImplementationOnce(
    constructGet({
      getUsers: async () => ({
        status: httpStatusCodes.OK,
        data: [defaultAdmin, user1],
      }),
    })
  )
  fireEvent.click(users.getByTestId("refresh-users-button"))
  await waitForDomChange()

  // Both user and admin should be in the table
  expect(users.getByText(defaultAdmin.email)).toBeInTheDocument()
  expect(users.getByText(user1.email)).toBeInTheDocument()
})
