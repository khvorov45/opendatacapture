import React from "react"
import Home from "./home"
import { render } from "@testing-library/react"
import { MemoryRouter, Switch, Route } from "react-router-dom"
import { Access, User } from "../lib/auth"

test("reroute to login", () => {
  const { getByTestId } = render(
    <MemoryRouter initialEntries={["/"]}>
      <Switch>
        <Route exact path="/">
          <Home
            token={null}
            tokenValidator={(t) =>
              new Promise((resolve, reject) =>
                resolve({
                  id: 1,
                  email: "email@example.com",
                  password_hash: "123",
                  access: Access.Admin,
                } as User)
              )
            }
          />
        </Route>
        <Route exact path="/login">
          <div data-testid="login-form" />
        </Route>
      </Switch>
    </MemoryRouter>
  )
  let loginForm = getByTestId("login-form")
  expect(loginForm).toBeInTheDocument()
})
