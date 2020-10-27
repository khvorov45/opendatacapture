import React from "react"
import { CircularProgress } from "@material-ui/core"
import { Redirect, Route, useLocation, useRouteMatch } from "react-router-dom"
import { SimpleNav } from "./nav"

export default function AdminDashboard({ token }: { token: string | null }) {
  const { pathname } = useLocation()
  return (
    <div data-testid="admin-dashboard">
      <SimpleNav
        links={["users", "all-projects"]}
        dataTestId="project-page-links"
        active={(l) => pathname.startsWith(`/admin/${l}`)}
      />
      {token ? <Main token={token} /> : <CircularProgress />}
    </div>
  )
}

function Main({ token }: { token: string }) {
  const { url } = useRouteMatch()
  return (
    <div>
      <Route exact path={url}>
        <Redirect to={`${url}/users`} />
      </Route>
      <Route path={`${url}/users`}>
        <Users token={token} />
      </Route>
      <Route path={`${url}/all-projects`}>
        <Projects token={token} />
      </Route>
    </div>
  )
}

function Users({ token }: { token: string }) {
  return <div data-testid="users-admin-widget">Users</div>
}

function Projects({ token }: { token: string }) {
  return <div data-testid="projects-admin-widget">Projects</div>
}
