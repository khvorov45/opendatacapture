import React from "react"
import { render, fireEvent } from "@testing-library/react"
import App from "./App"
import { theme_init } from "./lib/theme"

test("theme switching", () => {
  theme_init()
  const { getByTestId } = render(<App />)
  let themeswitch = getByTestId("themeswitch")
  expect(themeswitch).toBeInTheDocument()
  expect(localStorage.theme).toBe("dark")
  expect(document.documentElement.getAttribute("theme")).toBe("dark")
  fireEvent.click(themeswitch)
  expect(localStorage.theme).toBe("light")
  expect(document.documentElement.getAttribute("theme")).toBe("light")
})
