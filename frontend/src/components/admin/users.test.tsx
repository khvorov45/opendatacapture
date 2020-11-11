/* istanbul ignore file */
import axios from "axios"
import { fireEvent, render, wait } from "@testing-library/react"
import httpStatusCodes from "http-status-codes"
import { renderAdminPage } from "../../tests/util"
import { constructGet } from "../../tests/api"
import React from "react"
import Users from "./users"
import { act } from "react-dom/test-utils"

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
