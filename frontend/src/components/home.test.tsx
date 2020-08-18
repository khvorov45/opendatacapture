/* istanbul ignore file */
import React from "react"
import Home from "./home"
import { render, waitForElement } from "@testing-library/react"
import { MemoryRouter, Switch, Route } from "react-router-dom"
import { Access, User } from "../lib/auth"

async function tokenValidatorGood() {
  return {
    id: 1,
    email: "email@example.com",
    password_hash: "123",
    access: Access.Admin,
  } as User
}

async function tokenValidatorBad(): Promise<User> {
  throw Error()
}

function renderHome(
  token: string | null,
  tokenValidator: (s: string) => Promise<User>
) {
  return render(
    <MemoryRouter initialEntries={["/"]}>
      <Switch>
        <Route exact path="/">
          <Home token={token} tokenValidator={tokenValidator} />
        </Route>
        <Route exact path="/login">
          <div data-testid="login-form" />
        </Route>
      </Switch>
    </MemoryRouter>
  )
}

test("reroute to login with no token", async () => {
  const { getByTestId } = renderHome(null, tokenValidatorGood)
  const login = await waitForElement(() => getByTestId("login-form"))
  expect(login).toBeInTheDocument()
})

test("reroute to login with invalid token", async () => {
  const { getByTestId } = renderHome("123", tokenValidatorBad)
  const login = await waitForElement(() => getByTestId("login-form"))
  expect(login).toBeInTheDocument()
})

test("stay home when token is given and user is returned", async () => {
  const { getByTestId } = renderHome("123", tokenValidatorGood)
  const homepage = await waitForElement(() => getByTestId("homepage"))
  expect(homepage).toBeInTheDocument()
})
