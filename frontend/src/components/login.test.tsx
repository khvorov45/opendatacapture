import React from "react"
import { render, fireEvent, waitForDomChange } from "@testing-library/react"
import Login from "./login"
import { Token } from "../lib/auth"

test("login when can't connect to server", async () => {
  const { getByTestId } = render(
    <Login
      updateToken={(token: Token) => {}}
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
      updateToken={(token: Token) => {}}
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
    Error("EmailNotFound"),
    Error("WrongPassword"),
    Error("Network Error"),
  ]
  const { getByTestId } = render(
    <Login
      updateToken={(token: Token) => {}}
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
