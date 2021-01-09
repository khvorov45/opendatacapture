/* istanbul ignore file */
import React from "react"
import axios from "axios"
import httpStatusCodes from "http-status-codes"
import { render, fireEvent, waitFor } from "@testing-library/react"
import Login from "./login"
import { EmailPassword, Token } from "../lib/api/auth"
import { API_ROOT } from "../lib/config"
import { constructPost } from "../tests/api"

jest.mock("axios")
const mockedAxios = axios as jest.Mocked<typeof axios>

let postreq: any
beforeEach(() => {
  postreq = mockedAxios.post.mockImplementation(constructPost())
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
  await waitFor(() => {
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

  // Wrong email
  mockedAxios.post.mockImplementation(
    constructPost({
      fetchToken: async () => ({
        status: httpStatusCodes.UNAUTHORIZED,
        data: "NoSuchUserEmail",
      }),
    })
  )
  fireEvent.click(submitButton)
  await waitFor(() => {
    expect(submitButtonMsg.innerHTML).toBe("")
    verifyFieldError(emailField, "Email not found")
    verifyFieldError(passwordField, "")
  })

  // Wrong password
  mockedAxios.post.mockImplementation(
    constructPost({
      fetchToken: async () => ({
        status: httpStatusCodes.UNAUTHORIZED,
        data: "WrongPassword",
      }),
    })
  )
  fireEvent.click(submitButton)
  await waitFor(() => {
    expect(submitButtonMsg.innerHTML).toBe("")
    verifyFieldError(emailField, "")
    verifyFieldError(passwordField, "Wrong password")
  })

  // Unhandled axios error
  mockedAxios.post.mockImplementation(
    constructPost({
      fetchToken: async () => {
        throw Error("Network Error")
      },
    })
  )
  fireEvent.click(submitButton)
  await waitFor(() => {
    expect(submitButtonMsg.innerHTML).toBe("Network Error")
    verifyFieldError(emailField, "")
    verifyFieldError(passwordField, "")
  })
})
