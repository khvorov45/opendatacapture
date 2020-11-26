/* istanbul ignore file */
import React from "react"
import axios from "axios"
import httpStatusCodes from "http-status-codes"
import {
  render,
  fireEvent,
  waitForDomChange,
  wait,
} from "@testing-library/react"
import Login from "./login"
import { EmailPassword, LoginFailure, Token } from "../lib/api/auth"
import { API_ROOT } from "../lib/config"
import {
  constructGet,
  constructPut,
  constructDelete,
  constructPost,
} from "../tests/api"

jest.mock("axios")
const mockedAxios = axios as jest.Mocked<typeof axios>

const getreq = mockedAxios.get.mockImplementation(constructGet())
const putreq = mockedAxios.put.mockImplementation(constructPut())
const deletereq = mockedAxios.delete.mockImplementation(constructDelete())
const postreq = mockedAxios.post.mockImplementation(constructPost())
afterEach(() => {
  mockedAxios.get.mockImplementation(constructGet())
  mockedAxios.put.mockImplementation(constructPut())
  mockedAxios.delete.mockImplementation(constructDelete())
  mockedAxios.post.mockImplementation(constructPost())
})

function renderLogin(updateToken?: (t: Token) => void) {
  return render(<Login updateToken={updateToken ?? ((t) => {})} />)
}

test("login - basic functionality", async () => {
  let login = renderLogin()
  let emailInput = login.getByTestId("email-input") as HTMLInputElement
  let passwordInput = login.getByTestId("password-input") as HTMLInputElement
  let submitButton = login.getByText("Submit")
  let cred: EmailPassword = {
    email: "admin@example.com",
    password: "admin",
  }
  expect(emailInput.value).toBe("")
  fireEvent.change(emailInput, { target: { value: cred.email } })
  expect(emailInput.value).toBe(cred.email)
  expect(passwordInput.value).toBe("")
  fireEvent.change(passwordInput, { target: { value: cred.password } })
  expect(passwordInput.value).toBe(cred.password)
  fireEvent.click(submitButton)
  await wait(() => {
    expect(postreq).toHaveBeenCalledWith(
      `${API_ROOT}/auth/session-token`,
      cred,
      expect.anything()
    )
  })
})

function verifyFieldError(element: HTMLElement, expected: string) {
  let errorMsg = element.querySelector("p")
  if (expected !== "") {
    expect(errorMsg).toBeTruthy()
  }
  if (errorMsg) {
    expect(errorMsg.innerHTML).toBe(expected)
  }
}

test("login errors", async () => {
  const { getByTestId, getByText } = renderLogin()
  let emailField = getByTestId("email-field")
  let passwordField = getByTestId("password-field")
  let submitButton = getByText("Submit")
  let submitButtonMsg = getByTestId("login-button-msg")

  // Prior to any clicking
  expect(submitButtonMsg.innerHTML).toBe("")
  verifyFieldError(emailField, "")
  verifyFieldError(passwordField, "")

  mockedAxios.post
    .mockRejectedValueOnce(Error(LoginFailure.EmailNotFound))
    .mockRejectedValueOnce(Error(LoginFailure.WrongPassword))
    .mockRejectedValueOnce(Error("Network Error"))

  fireEvent.click(submitButton)
  await waitForDomChange()

  // Wrong email
  expect(submitButtonMsg.innerHTML).toBe("")
  verifyFieldError(emailField, "Email not found")
  verifyFieldError(passwordField, "")

  fireEvent.click(submitButton)
  await waitForDomChange()

  // Wrong password
  expect(submitButtonMsg.innerHTML).toBe("")
  verifyFieldError(emailField, "")
  verifyFieldError(passwordField, "Wrong password")

  fireEvent.click(submitButton)
  await waitForDomChange()

  // Network error
  expect(submitButtonMsg.innerHTML).toBe("Network Error")
  verifyFieldError(emailField, "")
  verifyFieldError(passwordField, "")
})
