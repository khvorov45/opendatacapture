import React from "react"
import { render, fireEvent, waitForDomChange } from "@testing-library/react"
import App from "./App"
import { themeInit } from "./lib/theme"

import axios from "axios"

jest.mock("axios")
const mockedAxios = axios as jest.Mocked<typeof axios>

function expectTheme(theme: "dark" | "light") {
  expect(localStorage.theme).toBe(theme)
  expect(document.documentElement.getAttribute("theme")).toBe(theme)
}

test("theme switching", () => {
  localStorage.removeItem("theme")
  themeInit()
  const { getByTestId } = render(<App initPalette="dark" initToken={null} />)
  let themeswitch = getByTestId("themeswitch")
  expect(themeswitch).toBeInTheDocument()
  expectTheme("dark")
  fireEvent.click(themeswitch)
  expectTheme("light")
  fireEvent.click(themeswitch)
  expectTheme("dark")
})

test("reroute to login", () => {
  const { getByTestId } = render(<App initPalette="dark" initToken={null} />)
  let loginForm = getByTestId("login-form")
  expect(loginForm).toBeInTheDocument()
})

test("token update", async () => {
  localStorage.removeItem("token")
  // Login response
  mockedAxios.post.mockResolvedValue({ status: 200, data: "123" })
  let user = {
    id: 1,
    email: "test@example.com",
    password_hash: "123",
    access: "Admin",
  }
  // Token validation response
  mockedAxios.get.mockResolvedValue({ data: user })
  const { getByTestId } = render(<App initPalette="dark" initToken={null} />)
  fireEvent.click(getByTestId("login-submit"))
  await waitForDomChange()
  expect(localStorage.getItem("token")).toBe("123")
})
