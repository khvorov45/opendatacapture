/* istanbul ignore file */
import React from "react"
import Home from "./home"
import { render } from "@testing-library/react"

function renderHome(token: string | null) {
  return render(<Home token={token} />)
}

test("homepage", () => {
  const { getByText } = renderHome("123")
  expect(getByText((s) => s.includes("123"))).toBeInTheDocument()
})
