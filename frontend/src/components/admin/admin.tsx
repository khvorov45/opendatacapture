import React from "react"
import {
  CircularProgress,
  createStyles,
  makeStyles,
  Theme,
} from "@material-ui/core"
import { Redirect, Route, useLocation, useRouteMatch } from "react-router-dom"
import { SimpleNav } from "../nav"
import Users from "./users"
import Projects from "./projects"

const useStyles = makeStyles((theme: Theme) =>
  createStyles({
    main: {
      display: "flex",
      justifyContent: "center",
    },
  })
)

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
  const classes = useStyles()
  return (
    <div className={classes.main}>
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
