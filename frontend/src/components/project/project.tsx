import { createStyles, Theme } from "@material-ui/core"
import { Redirect, useLocation, useRouteMatch } from "react-router-dom"
import makeStyles from "@material-ui/core/styles/makeStyles"
import React from "react"
import { Route, useParams } from "react-router-dom"
import TablePanel from "./table-panel"
import DataPanel from "./data-panel"
import { SimpleNav } from "../nav"

const useStyles = makeStyles((theme: Theme) =>
  createStyles({
    projectPage: {
      overflow: "auto",
      "& .hidden": {
        visibility: "hidden",
      },
      "& .nodisplay": {
        display: "none",
      },
    },
  })
)

export default function ProjectPage({ token }: { token: string | null }) {
  let { name } = useParams<{ name: string }>()
  const { pathname } = useLocation()
  const classes = useStyles()
  return (
    <div className={classes.projectPage} data-testid={`project-page-${name}`}>
      <SimpleNav
        links={["tables", "data"]}
        dataTestId="project-page-links"
        active={(l) => pathname.startsWith(`/project/${name}/${l}`)}
      />
      {token ? <Main token={token} /> : <></>}
    </div>
  )
}

function Main({ token }: { token: string }) {
  const { url } = useRouteMatch()
  const { name } = useParams<{ name: string }>()
  return (
    <main>
      <Route exact path={url}>
        <Redirect to={`${url}/tables`} />
      </Route>
      <Route path={`${url}/tables`}>
        <TablePanel token={token} projectName={name} />
      </Route>
      <Route path={`${url}/data`}>
        <DataPanel token={token} projectName={name} />
      </Route>
    </main>
  )
}
