/* istanbul ignore file */
import axios from "axios"
import {
  fireEvent,
  render,
  wait,
  waitForDomChange,
} from "@testing-library/react"
import httpStatusCodes from "http-status-codes"
import { defaultAdmin, user1Cred } from "../../tests/data"
import { constructDelete, constructGet, constructPut } from "../../tests/api"
import { user1 } from "../../tests/data"
import React from "react"
import Users from "./users"
import { API_ROOT } from "../../lib/config"

jest.mock("axios")
const mockedAxios = axios as jest.Mocked<typeof axios>

mockedAxios.get.mockImplementation(constructGet())
const mockedDelete = mockedAxios.delete.mockImplementation(constructDelete())
const mockedPut = mockedAxios.put.mockImplementation(constructPut())

afterEach(() => {
  mockedAxios.get.mockImplementation(constructGet())
  mockedAxios.delete.mockImplementation(constructDelete())
  mockedAxios.put.mockImplementation(constructPut())
})

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
  mockedAxios.get.mockImplementation(
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

test("delete user", async () => {
  // Make sure there is a user to delete
  mockedAxios.get.mockImplementation(
    constructGet({
      getUsers: async () => ({
        status: httpStatusCodes.OK,
        data: [defaultAdmin, user1],
      }),
    })
  )
  const users = renderUsers()
  await wait(() => {
    expect(users.getByTestId("users-admin-widget")).toBeInTheDocument()
  })
  expect(mockedDelete).not.toHaveBeenCalled()

  // Find the appropriate delete button
  const allDeleteBtn = users.getAllByTestId("remove-user")
  const lastDeleteBtn = allDeleteBtn[allDeleteBtn.length - 1]
  fireEvent.click(lastDeleteBtn)
  await waitForDomChange()

  expect(mockedDelete).toHaveBeenLastCalledWith(
    `${API_ROOT}/remove/user/${user1.email}`,
    expect.anything()
  )
})

test("create user", async () => {
  const users = renderUsers()
  await wait(() => {
    expect(users.getByTestId("users-admin-widget")).toBeInTheDocument()
  })
  fireEvent.click(users.getByTestId("open-new-user-form-button"))
  fireEvent.change(users.getByTestId("user-email-field"), {
    target: { value: user1Cred.email },
  })
  fireEvent.change(users.getByTestId("user-password-field"), {
    target: { value: user1Cred.password },
  })
  fireEvent.click(users.getByTestId("user-submit"))
  await waitForDomChange()
  expect(mockedPut).toHaveBeenLastCalledWith(
    `${API_ROOT}/create/user`,
    user1Cred,
    expect.anything()
  )
})
