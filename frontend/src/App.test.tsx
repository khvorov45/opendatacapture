/* istanbul ignore file */

import React from "react"
import {
  render,
  fireEvent,
  waitForDomChange,
  wait,
} from "@testing-library/react"
import App from "./App"
import { themeInit } from "./lib/theme"
import httpStatusCodes from "http-status-codes"
import axios from "axios"
import { Access, User } from "./lib/api/auth"

jest.mock("axios")
const mockedAxios = axios as jest.Mocked<typeof axios>

function renderApp(token?: string) {
  token
    ? localStorage.setItem("token", token)
    : localStorage.removeItem("token")
  return render(<App />)
}

function expectTheme(theme: "dark" | "light") {
  expect(localStorage.theme).toBe(theme)
  expect(document.documentElement.getAttribute("theme")).toBe(theme)
}

test("theme switching", () => {
  localStorage.removeItem("theme")
  themeInit()
  const { getByTestId } = renderApp()
  let themeswitch = getByTestId("themeswitch")
  expect(themeswitch).toBeInTheDocument()
  expectTheme("dark")
  fireEvent.click(themeswitch)
  expectTheme("light")
  fireEvent.click(themeswitch)
  expectTheme("dark")
})

test("route to homepage from login", async () => {
  localStorage.removeItem("token")
  // Login response
  mockedAxios.post.mockResolvedValueOnce({
    status: httpStatusCodes.OK,
    data: { user: 1, token: "123", created: new Date().toISOString() },
  })
  let user: User = {
    id: 1,
    email: "test@example.com",
    password_hash: "123",
    access: Access.Admin,
  }
  mockedAxios.get
    // Token validation response
    .mockResolvedValueOnce({
      status: httpStatusCodes.OK,
      data: user,
    })
    // Project list response
    .mockResolvedValueOnce({
      status: httpStatusCodes.OK,
      data: [],
    })
  // Attempt to render the homepage
  const { getByTestId } = renderApp()
  // Will only work if successfully redirected to login
  fireEvent.click(getByTestId("login-submit"))
  await waitForDomChange()
  expect(localStorage.getItem("token")).toBe("123")
  // Check that successful login redirects to homepage
  expect(getByTestId("homepage")).toBeInTheDocument()
})

test("reroute to login when token is wrong", async () => {
  const { getByTestId } = renderApp()
  expect(getByTestId("login-form")).toBeInTheDocument()
})

test("route to project page", async () => {
  let user = {
    id: 1,
    email: "test@example.com",
    password_hash: "123",
    access: "Admin",
  }
  let project = {
    user: 1,
    name: "somename",
    created: new Date().toISOString(),
  }
  mockedAxios.get
    // Token validation
    .mockResolvedValueOnce({ status: httpStatusCodes.OK, data: user })
    // Project list
    .mockResolvedValueOnce({ status: httpStatusCodes.OK, data: [project] })
    // Project metadata
    .mockResolvedValueOnce({ status: httpStatusCodes.OK, data: [] })
    // Project list
    .mockResolvedValueOnce({ status: httpStatusCodes.OK, data: [project] })
  const { getByTestId, getByText } = renderApp("123")
  await wait(() => {
    expect(getByText("somename")).toBeInTheDocument()
    expect(getByTestId("nav-project-info")).toHaveClass("nodisplay")
  })
  fireEvent.click(getByText("somename"))
  await wait(() => {
    // Check redirection
    expect(getByTestId("project-page-somename")).toBeInTheDocument()
    // Check that project info on nav updated
    expect(getByTestId("nav-project-info")).not.toHaveClass("nodisplay")
  })

  // Go back
  fireEvent.click(getByText("Projects"))
  await waitForDomChange()

  // Project info should disappear
  expect(getByTestId("nav-project-info")).toHaveClass("nodisplay")

  // We should be at the project list
  expect(getByTestId("homepage")).toBeInTheDocument()
  expect(getByTestId("home-link")).toHaveClass("active")
})

test("token refresh", async () => {
  const curTime = new Date().toISOString()
  mockedAxios.post.mockResolvedValueOnce({
    status: httpStatusCodes.OK,
    data: {
      user: 1,
      token: "234",
      created: curTime,
    },
  })
  localStorage.removeItem("last-refresh")
  const app = renderApp("123")
  await wait(() => {
    expect(localStorage.getItem("last-refresh")).toBe(curTime)
  })
  expect(localStorage.getItem("token")).toBe("234")
})
