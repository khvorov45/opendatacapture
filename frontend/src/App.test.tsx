import React from "react"
import { render, fireEvent } from "@testing-library/react"
import App from "./App"
import { themeInit } from "./lib/theme"

test("theme switching", () => {
  themeInit()
  const { getByTestId } = render(<App initPalette="dark" />)
  let themeswitch = getByTestId("themeswitch")
  expect(themeswitch).toBeInTheDocument()
  expect(localStorage.theme).toBe("dark")
  expect(document.documentElement.getAttribute("theme")).toBe("dark")
  fireEvent.click(themeswitch)
  expect(localStorage.theme).toBe("light")
  expect(document.documentElement.getAttribute("theme")).toBe("light")
})
