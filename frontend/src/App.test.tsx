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
import { API_ROOT } from "./lib/config"
import {
  constructDelete,
  constructGet,
  constructPost,
  constructPut,
  defaultPost,
} from "./tests/api"

jest.mock("axios")
const mockedAxios = axios as jest.Mocked<typeof axios>

const postreq = mockedAxios.post.mockImplementation(constructPost())
const getreq = mockedAxios.get.mockImplementation(constructGet())
const putreq = mockedAxios.put.mockImplementation(constructPut())
const deletereq = mockedAxios.delete.mockImplementation(constructDelete())
afterEach(() => {
  mockedAxios.get.mockImplementation(constructGet())
  mockedAxios.put.mockImplementation(constructPut())
  mockedAxios.delete.mockImplementation(constructDelete())
  mockedAxios.post.mockImplementation(constructPost())
})

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
  // Attempt to render the homepage
  const { getByTestId } = renderApp()
  // Will only work if successfully redirected to login
  fireEvent.click(getByTestId("login-submit"))
  await waitForDomChange()
  expect(localStorage.getItem("token")).toBe(
    (await defaultPost.fetchToken()).data.token
  )
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
  // Token verification
  mockedAxios.get.mockResolvedValueOnce({
    status: httpStatusCodes.OK,
    data: {
      id: 1,
      email: "test@example.com",
      password_hash: "123",
      access: "Admin",
    },
  })
  localStorage.removeItem("last-refresh")
  const app = renderApp("123")
  await wait(() => {
    expect(localStorage.getItem("last-refresh")).toBe(curTime)
  })
  expect(localStorage.getItem("token")).toBe("234")
})

test("token refresh error", async () => {
  const consoleSpy = jest
    .spyOn(console, "error")
    .mockImplementation((message) => {})
  // Token verification
  mockedAxios.get.mockResolvedValueOnce({
    status: httpStatusCodes.OK,
    data: {
      id: 1,
      email: "test@example.com",
      password_hash: "123",
      access: "Admin",
    },
  })
  mockedAxios.post.mockRejectedValueOnce(Error("some refresh error"))
  localStorage.removeItem("last-refresh")
  const app = renderApp("123")
  await wait(() => {
    expect(console.error).toHaveBeenLastCalledWith("some refresh error")
  })
  consoleSpy.mockRestore()
})

test("logout", async () => {
  const consoleSpy = jest
    .spyOn(console, "error")
    .mockImplementation((message) => {})
  // Token verification
  mockedAxios.get.mockResolvedValueOnce({
    status: httpStatusCodes.OK,
    data: {
      id: 1,
      email: "test@example.com",
      password_hash: "123",
      access: "Admin",
    },
  })
  // Token removal
  const del = mockedAxios.delete.mockImplementationOnce(async () => {})
  localStorage.setItem("last-refresh", new Date().toISOString())
  const app = renderApp("123")
  await wait(() => {
    expect(app.getByTestId("homepage")).toBeInTheDocument()
  })
  const logout = app.getByTestId("logout-button")
  expect(logout).not.toHaveClass("nodisplay")
  fireEvent.click(logout)
  expect(logout).toHaveClass("nodisplay")
  expect(app.getByTestId("login-form")).toBeInTheDocument()
  expect(del).toHaveBeenLastCalledWith(
    `${API_ROOT}/auth/remove-token/123`,
    expect.anything()
  )
  consoleSpy.mockRestore()
})

test("fail to remove token", async () => {
  const consoleSpy = jest.spyOn(console, "error").mockImplementation(() => {})
  mockedAxios.delete.mockRejectedValue(Error("some delete token error"))
  // Token verification
  mockedAxios.get.mockResolvedValueOnce({
    status: httpStatusCodes.OK,
    data: {
      id: 1,
      email: "test@example.com",
      password_hash: "123",
      access: "Admin",
    },
  })
  localStorage.setItem("last-refresh", new Date().toISOString())
  const app = renderApp("123")
  await wait(() => {
    expect(app.getByTestId("homepage")).toBeInTheDocument()
  })
  fireEvent.click(app.getByTestId("logout-button"))
  await wait(() => {
    expect(consoleSpy).toHaveBeenLastCalledWith("some delete token error")
  })
  // If we somehow manage to logout with no current token then the api call
  // shouldn't happen
  expect(consoleSpy).toHaveBeenCalledTimes(1)
  localStorage.setItem("last-refresh", new Date().toISOString())
  fireEvent.click(app.getByTestId("logout-button"))
  await wait(() => {
    expect(localStorage.getItem("last-refresh")).toBeNull()
  })
  expect(consoleSpy).toHaveBeenCalledTimes(1)
  consoleSpy.mockRestore()
})

test("admin dashboard access", async () => {
  mockedAxios.get.mockResolvedValueOnce({
    status: httpStatusCodes.OK,
    data: {
      id: 1,
      email: "test@example.com",
      access: "Admin",
    },
  })
  localStorage.setItem("last-refresh", new Date().toISOString())
  const app = renderApp("123")
  await wait(() => {
    expect(app.getByTestId("homepage")).toBeInTheDocument()
  })
  const adminLink = app.getByText("Admin")
  expect(adminLink.parentElement).not.toHaveClass("nodisplay")
  fireEvent.click(adminLink)
  expect(app.getByTestId("admin-dashboard")).toBeInTheDocument()
  // It somehow remembers the last postion on the next render, so better go back
  fireEvent.click(app.getByText("Projects"))
  await wait(() => {
    expect(app.getByTestId("homepage")).toBeInTheDocument()
  })
})

test("admin dashboard user no access", async () => {
  mockedAxios.get.mockResolvedValueOnce({
    status: httpStatusCodes.OK,
    data: {
      id: 1,
      email: "test@example.com",
      access: "User",
    },
  })
  localStorage.setItem("last-refresh", new Date().toISOString())
  const app = renderApp("123")
  await wait(() => {
    expect(app.getByTestId("homepage")).toBeInTheDocument()
  })
  const adminLink = app.getByText("Admin")
  expect(adminLink.parentElement).toHaveClass("nodisplay")
  fireEvent.click(adminLink)
  expect(app.queryByTestId("admin-dashboard")).not.toBeInTheDocument()
  fireEvent.click(app.getByText("Projects"))
  await wait(() => {
    expect(app.getByTestId("homepage")).toBeInTheDocument()
  })
})
