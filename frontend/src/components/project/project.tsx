import { createStyles, Theme } from "@material-ui/core"
import { Redirect, useRouteMatch } from "react-router-dom"
import makeStyles from "@material-ui/core/styles/makeStyles"
import React from "react"
import { Route, useParams } from "react-router-dom"
import { ButtonLink } from "../button"
import TablePanel from "./table-panel"

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
    sidebar: {
      backgroundColor: "var(--palette-bg-alt)",
      borderBottom: `1px solid ${theme.palette.divider}`,
    },
  })
)

export default function ProjectPage({ token }: { token: string | null }) {
  let { name } = useParams<{ name: string }>()
  const classes = useStyles()
  return (
    <div className={classes.projectPage} data-testid={`project-page-${name}`}>
      <Sidebar />
      {token ? <Main token={token} /> : <></>}
    </div>
  )
}

function Sidebar() {
  const { url } = useRouteMatch()
  const classes = useStyles()
  return (
    <div className={classes.sidebar} data-testid="project-page-links">
      <ButtonLink active={true} to={`${url}/tables`}>
        Tables
      </ButtonLink>
    </div>
  )
}

function Main({ token }: { token: string }) {
  const { url } = useRouteMatch()
  const { name } = useParams<{ name: string }>()
  return (
    <main>
      <Route path={url}>
        <Redirect to={`${url}/tables`} />
      </Route>
      <Route path={`${url}/tables`}>
        <TablePanel token={token} projectName={name} />
      </Route>
    </main>
  )
}
