import React from "react"
import {
  render,
  fireEvent,
  waitForDomChange,
  wait,
} from "@testing-library/react"
import Login from "./login"
import { BrowserRouter, Switch, Route, Redirect } from "react-router-dom"
import { LoginFailure } from "../lib/auth"

test("basic functionality", async () => {
  let token: string | null = null
  let token_expected = "123"
  let { getByTestId } = render(
    <BrowserRouter>
      <Switch>
        <Route path="/login">
          <Login
            updateToken={(t) => (token = t)}
            tokenFetcher={(c) =>
              new Promise((resolve, reject) => resolve(token_expected))
            }
          />
        </Route>
        <Route path="/">
          <Redirect to="/login" />
        </Route>
      </Switch>
    </BrowserRouter>
  )
  let emailInput = getByTestId("email-input") as HTMLInputElement
  let passwordInput = getByTestId("password-input") as HTMLInputElement
  let submitButton = getByTestId("login-submit")
  expect(emailInput.value).toBe("")
  fireEvent.change(emailInput, { target: { value: "admin@example.com" } })
  expect(emailInput.value).toBe("admin@example.com")
  expect(passwordInput.value).toBe("")
  fireEvent.change(passwordInput, { target: { value: "admin" } })
  expect(passwordInput.value).toBe("admin")
  expect(token).toBe(null)
  fireEvent.click(submitButton)
  await wait(() => {
    expect(token).toBe(token_expected)
  })
})

test("login when can't connect to server", async () => {
  const { getByTestId } = render(
    <Login
      updateToken={(token: string) => {}}
      tokenFetcher={(c) =>
        new Promise((resolve, reject) => reject(Error("Network Error")))
      }
    />
  )
  let submitButton = getByTestId("login-submit")
  let submitButtonMsg = getByTestId("login-button-msg")
  expect(submitButtonMsg.innerHTML).toBe("")
  spyOn(console, "error") // There is expected to be an error
  fireEvent.click(submitButton)
  await waitForDomChange()
  expect(submitButtonMsg.innerHTML).toBe("Network Error")
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

test("login when email is wrong", async () => {
  const { getByTestId } = render(
    <Login
      updateToken={(token: string) => {}}
      tokenFetcher={(c) =>
        new Promise((resolve, reject) => reject(Error("EmailNotFound")))
      }
    />
  )
  let emailField = getByTestId("email-field")
  let submitButton = getByTestId("login-submit")
  spyOn(console, "error") // There is expected to be an error
  fireEvent.click(submitButton)
  await waitForDomChange()
  verifyFieldError(emailField, "Email not found")
})

test("errors reset", async () => {
  let rejectIndex = 0
  let rejections = [
    Error(LoginFailure.EmailNotFound),
    Error(LoginFailure.WrongPassword),
    Error("Network Error"),
  ]
  const { getByTestId } = render(
    <Login
      updateToken={(token: string) => {}}
      tokenFetcher={(c) =>
        new Promise((resolve, reject) => reject(rejections[rejectIndex++]))
      }
    />
  )
  let emailField = getByTestId("email-field")
  let passwordField = getByTestId("password-field")
  let submitButton = getByTestId("login-submit")
  let submitButtonMsg = getByTestId("login-button-msg")
  spyOn(console, "error") // There is expected to be an error

  // Prior to any clicking
  expect(submitButtonMsg.innerHTML).toBe("")
  verifyFieldError(emailField, "")
  verifyFieldError(passwordField, "")

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
