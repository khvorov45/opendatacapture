import {
  createStyles,
  List,
  ListItem,
  ListItemText,
  Theme,
} from "@material-ui/core"
import makeStyles from "@material-ui/core/styles/makeStyles"
import React, { useEffect, useState } from "react"
import { useParams } from "react-router-dom"
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

export default function ProjectPage({
  onVisit,
}: {
  onVisit: (projectName: string) => void
}) {
  let { name } = useParams<{ name: string }>()
  useEffect(() => {
    onVisit?.(name)
  }, [name, onVisit])
  let [tableSpec, setTableSpec] = useState<TableSpec>([])
  const classes = useStyles()
  return (
    <div className={classes.projectPage} data-testid={`project-page-${name}`}>
      <Sidebar />
      <main className={classes.main}>
        <div>main</div>
        <TableCards tableSpec={tableSpec} />
      </main>
    </div>
  )
}

function Sidebar() {
  const classes = useStyles()
  return (
    <div className={classes.sidebar}>
      <List>
        <ListItem button>
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
