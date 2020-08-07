import React from "react"
import { render, fireEvent, waitForDomChange } from "@testing-library/react"
import Login from "./login"
import { Token } from "../lib/auth"

test("login when server offline", async () => {
  const { getByTestId } = render(<Login updateToken={(token: Token) => {}} />)
  let emailInput = getByTestId("email-input")
  let passwordInput = getByTestId("password-input")
  let submitButton = getByTestId("login-submit")
  let submitButtonMsg = getByTestId("login-button-msg")
  fireEvent.change(emailInput, { target: { value: "admin@example.com" } })
  fireEvent.change(passwordInput, { target: { value: "admin" } })
  expect(submitButtonMsg.innerHTML).toBe("")
  spyOn(console, "error") // There is expected to be an error
  fireEvent.click(submitButton)
  await waitForDomChange()
  expect(submitButtonMsg.innerHTML).toBe("Network Error")
})
