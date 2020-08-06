import { themeInit } from "./theme"

function expectTheme(theme: "dark" | "light") {
  expect(themeInit()).toBe(theme)
  expect(localStorage.getItem("theme")).toBe(theme)
  expect(document.documentElement.getAttribute("theme")).toBe(theme)
}

test("themeInit", () => {
  localStorage.removeItem("theme")
  expectTheme("dark")
  localStorage.setItem("theme", "light")
  expectTheme("light")
})
