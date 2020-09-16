import {
  createStyles,
  List,
  ListItem,
  ListItemText,
  Theme,
} from "@material-ui/core"
import { Link, Redirect } from "react-router-dom"
import makeStyles from "@material-ui/core/styles/makeStyles"
import React, { useState } from "react"
import { Route, useParams } from "react-router-dom"
import { TableMeta, TableSpec } from "../lib/project"

const useStyles = makeStyles((theme: Theme) =>
  createStyles({
    projectPage: {
      display: "grid",
      overflow: "auto",
      gridTemplateColumns: "[sidebar] 1fr [main] 8fr",
    },
    sidebar: {
      gridColumnStart: "sidebar",
      backgroundColor: "var(--palette-sidebar)",
    },
    main: {
      gridColumnStart: "main",
    },
  })
)

export default function ProjectPage() {
  let { name } = useParams<{ name: string }>()

  let [tableSpec, setTableSpec] = useState<TableSpec>([])
  const classes = useStyles()
  return (
    <div className={classes.projectPage} data-testid={`project-page-${name}`}>
      <Sidebar projectName={name} />
      <main className={classes.main}>
        <div>main</div>
        <Route path={`/project/${name}`}>
          <Redirect to={`/project/${name}/tables`} />
        </Route>
        <Route path={`/project/${name}/tables`}>
          <div>tables</div>
          <TableCards tableSpec={tableSpec} />
        </Route>
      </main>
    </div>
  )
}

function Sidebar({ projectName }: { projectName: string }) {
  const classes = useStyles()
  return (
    <div className={classes.sidebar}>
      <List>
        <ListItem button component={Link} to={`/project/${projectName}/tables`}>
          <ListItemText primary="Tables" />
        </ListItem>
      </List>
    </div>
  )
}

function TableCards({ tableSpec }: { tableSpec: TableSpec }) {
  return (
    <>
      {tableSpec.map((tableMeta) => (
        <TableCard tableMeta={tableMeta} />
      ))}
    </>
  )
}

function TableCard({ tableMeta }: { tableMeta: TableMeta }) {
  return <>Table card for {tableMeta.name}</>
}
