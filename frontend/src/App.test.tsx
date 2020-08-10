import React from "react"
import { render, fireEvent } from "@testing-library/react"
import App from "./App"
import { themeInit } from "./lib/theme"
import { MemoryRouter } from "react-router-dom"

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
})

test("reroute to login", () => {
  const { getByTestId } = render(
    <MemoryRouter initialEntries={["/"]}>
      <App initPalette="dark" initToken={null} />
    </MemoryRouter>
  )
  let loginForm = getByTestId("login-form")
  expect(loginForm).toBeInTheDocument()
})
