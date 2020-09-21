import {
  createStyles,
  Divider,
  List,
  ListItem,
  ListItemText,
  TextField,
  Theme,
} from "@material-ui/core"
import { Link, Redirect, useRouteMatch } from "react-router-dom"
import makeStyles from "@material-ui/core/styles/makeStyles"
import React, { useCallback, useEffect, useState } from "react"
import { Route, useParams } from "react-router-dom"
import { ColMeta, getAllMeta, TableMeta, TableSpec } from "../lib/project"
import { ButtonArray, CreateButton } from "./button"

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
      borderRight: `1px solid ${theme.palette.divider}`,
    },
    main: {
      gridColumnStart: "main",
    },
    tableControl: {
      borderBottom: `1px solid ${theme.palette.divider}`,
    },
    tableCards: {
      display: "flex",
      "&>*": {
        borderRight: `1px solid ${theme.palette.divider}`,
        borderBottom: `1px solid ${theme.palette.divider}`,
      },
    },
    newTableForm: {
      display: "flex",
      flexDirection: "column",
      maxWidth: "200px",
      padding: "10px",
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
    <div className={classes.sidebar}>
      <List>
        <ListItem button component={Link} to={`${url}/tables`}>
          <ListItemText primary="Tables" />
        </ListItem>
      </List>
    </div>
  )
}

function Main({ token }: { token: string }) {
  const { url } = useRouteMatch()
  const { name } = useParams<{ name: string }>()
  const classes = useStyles()
  return (
    <main className={classes.main}>
      <Route path={url}>
        <Redirect to={`${url}/tables`} />
      </Route>
      <Route path={`${url}/tables`}>
        <TablePanel token={token} projectName={name} />
      </Route>
    </main>
  )
}

function TablePanel({
  token,
  projectName,
}: {
  token: string
  projectName: string
}) {
  let [renderNew, setRenderNew] = useState(false)
  let [tableSpec, setTableSpec] = useState<TableSpec>([])
  let [errorMsg, setErrorMsg] = useState("")

  useEffect(() => {
    if (tableSpec.length === 0) {
      setRenderNew(true)
    }
  }, [tableSpec])

  const refreshTables = useCallback(() => {
    getAllMeta(token, projectName)
      .then((tables) => setTableSpec(tables))
      .catch((e) => setErrorMsg(e.message))
  }, [token, projectName])

  useEffect(() => {
    refreshTables()
  }, [refreshTables])

  return (
    <>
      <TableControl onCreate={() => setRenderNew((old) => !old)} />
      <TableCards tableSpec={tableSpec} renderNew={renderNew} />
    </>
  )
}

function TableControl({ onCreate }: { onCreate: () => void }) {
  const classes = useStyles()
  return (
    <ButtonArray className={classes.tableControl}>
      <CreateButton onClick={onCreate} dataTestId="create-table-button" />
    </ButtonArray>
  )
}

function TableCards({
  tableSpec,
  renderNew,
}: {
  tableSpec: TableSpec
  renderNew: boolean
}) {
  const classes = useStyles()
  return (
    <div className={classes.tableCards}>
      {renderNew ? <NewTableForm /> : <></>}
      {tableSpec.map((tableMeta) => (
        <TableCard key={tableMeta.name} tableMeta={tableMeta} />
      ))}
    </div>
  )
}

function TableCard({ tableMeta }: { tableMeta: TableMeta }) {
  return <>Table card for {tableMeta.name}</>
}

function NewTableForm() {
  const [name, setName] = useState("")
  const defaultCol = {
    name: "",
    postgres_type: "",
    not_null: false,
    unique: false,
    primary_key: false,
    foreign_key: null,
  }
  const [cols, setCols] = useState<ColMeta[]>([defaultCol])

  function setColName(value: string, i: number) {
    // This is extremely memory-efficient, I swear
    const newCols = [...cols]
    newCols[i].name = value
    setCols(newCols)
  }
  const classes = useStyles()
  return (
    <div className={classes.newTableForm} data-testid="new-table-form">
      <TextField
        inputProps={{ "data-testid": "new-table-name-field" }}
        label="New table name..."
        value={name}
        onChange={(e) => setName(e.target.value)}
      />
      {cols.map((c, i) => (
        <ColumnEntry key={i} onNameChange={(value) => setColName(value, i)} />
      ))}
    </div>
  )
}

function ColumnEntry({
  onNameChange,
}: {
  onNameChange: (value: string) => void
}) {
  return (
    <div>
      <TextField
        inputProps={{ "data-testid": "new-column-name-field" }}
        label="New column name..."
        onChange={(e) => {
          onNameChange(e.target.value)
        }}
      />
    </div>
  )
}
