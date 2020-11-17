/* istanbul ignore file */
import React from "react"
import { render } from "@testing-library/react"
import { Redirect, Route, Switch, MemoryRouter } from "react-router-dom"
import ProjectPage from "../components/project/project"

export function renderProjectPage(
  token?: string | null,
  path?: "tables" | "data"
) {
  let tok: string | null = "123"
  if (token !== undefined) {
    tok = token
  }
  return render(
    <MemoryRouter
      initialEntries={[path ? `/project/some-project/${path}` : "/"]}
    >
      <Switch>
        <Route exact path="/">
          <Redirect to="/project/some-project" />
        </Route>
        <Route path="/project/:name">
          <ProjectPage token={tok} />
        </Route>
      </Switch>
    </MemoryRouter>
  )
}
