/* istanbul ignore file */
import React from "react"
import Home from "./home"
import { render } from "@testing-library/react"

function renderHome(token: string | null) {
  return render(<Home token={token} />)
}

test("homepage", () => {
  const { getByTestId } = renderHome("123")
  expect(getByTestId("project-list")).toBeInTheDocument()
})
