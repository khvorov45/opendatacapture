/* istanbul ignore file */

import React from "react"
import { render, fireEvent, waitForDomChange } from "@testing-library/react"
import App from "./App"
import { themeInit } from "./lib/theme"
import httpStatusCodes from "http-status-codes"
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

test("token update", async () => {
  localStorage.removeItem("token")
  // Login response
  mockedAxios.post.mockResolvedValueOnce({
    status: httpStatusCodes.OK,
    data: "123",
  })
  let user = {
    id: 1,
    email: "test@example.com",
    password_hash: "123",
    access: "Admin",
  }
  // Token validation response
  mockedAxios.get.mockResolvedValueOnce({
    status: httpStatusCodes.OK,
    data: user,
  })
  // Project list response
  mockedAxios.get.mockResolvedValueOnce({
    status: httpStatusCodes.OK,
    data: [],
  })
  // Attempt to render the homepage
  const { getByTestId } = render(<App initPalette="dark" initToken={null} />)
  // Will only work if successfully redirected to login
  fireEvent.click(getByTestId("login-submit"))
  await waitForDomChange()
  expect(localStorage.getItem("token")).toBe("123")
  // Check that successful login redirects to homepage
  expect(getByTestId("homepage")).toBeInTheDocument()
})

test("reroute to login when token is wrong", async () => {
  mockedAxios.get.mockRejectedValueOnce(Error(""))
  const { getByTestId } = render(<App initPalette="dark" initToken="123" />)
  await waitForDomChange()
  expect(getByTestId("login-form")).toBeInTheDocument()
})
