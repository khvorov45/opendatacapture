import {
  Checkbox as MaterialCheckbox,
  createStyles,
  FormControl,
  FormControlLabel,
  IconButton,
  InputLabel,
  MenuItem,
  Select as MaterialSelect,
  TextField,
  Theme,
  useTheme,
} from "@material-ui/core"
import { Redirect, useRouteMatch } from "react-router-dom"
import makeStyles from "@material-ui/core/styles/makeStyles"
import React, {
  ChangeEvent,
  ReactNode,
  useCallback,
  useEffect,
  useState,
} from "react"
import { Route, useParams } from "react-router-dom"
import {
  ColMeta,
  createTable,
  ForeignKey,
  getAllMeta,
  TableMeta,
  TableSpec,
} from "../lib/project"
import { ButtonArray, ButtonLink, CreateButton, RefreshButton } from "./button"
import Check from "@material-ui/icons/Check"
import Clear from "@material-ui/icons/Clear"
import Remove from "@material-ui/icons/Remove"
import { NamedDivider } from "./divider"
import { trackPromise, usePromiseTracker } from "react-promise-tracker"

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
      backgroundColor: "var(--palette-sidebar)",
      borderRight: `1px solid ${theme.palette.divider}`,
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
      width: "350px",
      "&>.padded": {
        margin: "auto",
        paddingLeft: "10px",
        paddingRight: "10px",
        paddingBottom: "10px",
      },
    },
    columnEntry: {
      display: "flex",
      flexDirection: "column",
      "&>*": {
        display: "flex",
      },
      "&>.delete": {
        justifyContent: "flex-end",
      },
      "&>*>*": {
        marginRight: 16,
      },
      "&>*>*:last-child": {
        marginRight: 0,
      },
    },
    select: {
      minWidth: 80,
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

  const { promiseInProgress } = usePromiseTracker({ area: "getAllMeta" })
  const refreshTables = useCallback(() => {
    trackPromise(getAllMeta(token, projectName), "getAllMeta")
      .then((tables) => setTableSpec(tables))
      .catch((e) => setErrorMsg(e.message))
  }, [token, projectName])

  useEffect(() => {
    refreshTables()
  }, [refreshTables])

  const classes = useStyles()
  return (
    <>
      <ButtonArray className={classes.tableControl} errorMsg={errorMsg}>
        <CreateButton
          onClick={() => setRenderNew((old) => !old)}
          dataTestId="create-table-button"
        />
        <RefreshButton onClick={refreshTables} inProgress={promiseInProgress} />
      </ButtonArray>
      <TableCards
        tableSpec={tableSpec}
        renderNew={renderNew}
        token={token}
        projectName={projectName}
        onSubmitNew={refreshTables}
      />
    </>
  )
}

function TableCards({
  tableSpec,
  renderNew,
  token,
  projectName,
  onSubmitNew,
}: {
  tableSpec: TableSpec
  renderNew: boolean
  token: string
  projectName: string
  onSubmitNew: () => void
}) {
  const classes = useStyles()
  return (
    <div className={classes.tableCards}>
      {renderNew ? (
        <NewTableForm
          token={token}
          projectName={projectName}
          onSubmit={onSubmitNew}
        />
      ) : (
        <></>
      )}
      {tableSpec.map((tableMeta) => (
        <TableCard key={tableMeta.name} tableMeta={tableMeta} />
      ))}
    </div>
  )
}

function TableCard({ tableMeta }: { tableMeta: TableMeta }) {
  return <>Table card for {tableMeta.name}</>
}

function NewTableForm({
  token,
  projectName,
  onSubmit,
}: {
  token: string
  projectName: string
  onSubmit: () => void
}) {
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
    const newCols = [...cols]
    newCols[i].name = value
    setCols(newCols)
  }
  function setColType(value: string, i: number) {
    const newCols = [...cols]
    newCols[i].postgres_type = value
    setCols(newCols)
  }
  function setColPK(value: boolean, i: number) {
    const newCols = [...cols]
    newCols[i].primary_key = value
    setCols(newCols)
  }
  function setColNN(value: boolean, i: number) {
    const newCols = [...cols]
    newCols[i].not_null = value
    setCols(newCols)
  }
  function setColUnique(value: boolean, i: number) {
    const newCols = [...cols]
    newCols[i].unique = value
    setCols(newCols)
  }
  function setColForeignKey(value: ForeignKey | null, i: number) {
    const newCols = [...cols]
    newCols[i].foreign_key = value
    setCols(newCols)
  }
  function addCol() {
    const newCols = [...cols]
    newCols.push(defaultCol)
    setCols(newCols)
  }

  const [removed, setRemoved] = useState<number[]>([])
  function removeCol(i: number) {
    const newRemoved = [...removed]
    newRemoved.push(i)
    setRemoved(newRemoved)
  }

  function handleClear() {
    setErrorMsg("")
    setName("")
    setCols([])
  }

  const [errorMsg, setErrorMsg] = useState("")
  function handleSubmit() {
    const tableMeta = {
      name: name,
      cols: cols.filter((c, i) => !removed.includes(i)),
    }
    createTable(token, projectName, tableMeta)
      .then(() => {
        handleClear()
        onSubmit()
      })
      .catch((e) => setErrorMsg(e.message))
  }

  const classes = useStyles()
  const theme = useTheme()
  return (
    <div className={classes.newTableForm} data-testid="new-table-form">
      <div className="padded">
        <TextField
          inputProps={{ "data-testid": "new-table-name-field" }}
          label="Table name"
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
      </div>
      <NamedDivider name="Columns" />
      <div className="padded">
        {cols.map((c, i) => (
          <ColumnEntry
            key={i}
            onNameChange={(value) => setColName(value, i)}
            onTypeChange={(value) => setColType(value, i)}
            onPKChange={(value) => setColPK(value, i)}
            onNNChange={(value) => setColNN(value, i)}
            onUniqueChange={(value) => setColUnique(value, i)}
            onFKChange={(value) => setColForeignKey(value, i)}
            onRemove={() => removeCol(i)}
          />
        ))}
      </div>
      <div>
        <CreateButton onClick={addCol} />
      </div>
      <NamedDivider name="" />
      <ButtonArray center className={"buttons"} errorMsg={errorMsg}>
        <IconButton onClick={handleSubmit} data-testid="submit-table-button">
          <Check htmlColor={theme.palette.success.main} />
        </IconButton>
        <IconButton onClick={handleClear} data-testid="clear-table-button">
          <Clear htmlColor={theme.palette.error.main} />
        </IconButton>
      </ButtonArray>
    </div>
  )
}

function ColumnEntry({
  onNameChange,
  onTypeChange,
  onPKChange,
  onNNChange,
  onUniqueChange,
  onFKChange,
  onRemove,
}: {
  onNameChange: (value: string) => void
  onTypeChange: (value: string) => void
  onPKChange: (value: boolean) => void
  onNNChange: (value: boolean) => void
  onUniqueChange: (value: boolean) => void
  onFKChange: (value: ForeignKey | null) => void
  onRemove: () => void
}) {
  const allowedTypes = ["int", "text"]
  const [type, setType] = useState("")
  function handleTypeChange(newType: string) {
    setType(newType)
    onTypeChange(newType)
  }
  const [primaryKey, setPrimaryKey] = useState(false)
  function handlePKChange(newPK: boolean) {
    setPrimaryKey(newPK)
    onPKChange(newPK)
  }
  const [notNull, setNotNull] = useState(false)
  function handleNNChange(newNN: boolean) {
    setNotNull(newNN)
    onNNChange(newNN)
  }
  const [unique, setUnique] = useState(false)
  function handleUniqueChange(newU: boolean) {
    setUnique(newU)
    onUniqueChange(newU)
  }
  const [foreignKey, setForeignKey] = useState(false)
  function handleFKChange(newFK: boolean) {
    setForeignKey(newFK)
    if (!newFK) {
      onFKChange(null)
    }
  }
  const [foreignTable, setForeignTable] = useState("")
  function handleForeignTableChange(newTable: string) {
    setForeignTable(newTable)
    onFKChange({ table: newTable, column: foreignColumn })
  }
  const [foreignColumn, setForeignColumn] = useState("")
  function handleForeignColumnChange(newCol: string) {
    setForeignColumn(newCol)
    onFKChange({ table: foreignTable, column: newCol })
  }
  const [removed, setRemoved] = useState(false)
  function handleRemove() {
    setRemoved(true)
    onRemove()
  }
  const classes = useStyles()
  const theme = useTheme()
  return (
    <div className={`${classes.columnEntry}${removed ? " nodisplay" : ""}`}>
      <div>
        <TextField
          inputProps={{ "data-testid": "new-column-name-field" }}
          label="Name"
          onChange={(e) => {
            onNameChange(e.target.value)
          }}
        />
        <Select id="type" value={type} onChange={handleTypeChange} label="Type">
          {allowedTypes.map((t) => (
            <MenuItem key={t} value={t}>
              {t}
            </MenuItem>
          ))}
        </Select>
      </div>

      <div>
        <Checkbox
          checked={primaryKey}
          onChange={handlePKChange}
          label="Primary key"
        />
        <Checkbox
          checked={notNull}
          onChange={handleNNChange}
          label="Not null"
          hidden={primaryKey}
        />
        <Checkbox
          checked={unique}
          onChange={handleUniqueChange}
          label="Unique"
          hidden={primaryKey}
        />
      </div>

      <div>
        <Checkbox
          checked={foreignKey}
          onChange={handleFKChange}
          label="Foreign key"
        />
        <Select
          id="fk-table"
          value={foreignTable}
          onChange={handleForeignTableChange}
          label="Table"
          hidden={!foreignKey}
        >
          <MenuItem value="a">A</MenuItem>
        </Select>
        <Select
          id="fk-column"
          value={foreignColumn}
          onChange={handleForeignColumnChange}
          label="Column"
          hidden={!foreignKey}
        >
          <MenuItem value="a">C</MenuItem>
        </Select>
      </div>
      <div className="delete">
        <IconButton onClick={(e) => handleRemove()}>
          <Remove htmlColor={theme.palette.error.main} />
        </IconButton>
      </div>
    </div>
  )
}

function Select({
  children,
  value,
  onChange,
  id,
  label,
  hidden,
}: {
  children: ReactNode
  value: string
  onChange: (value: string) => void
  id: string
  label: string
  hidden?: boolean
}) {
  const classes = useStyles()
  return (
    <FormControl className={`${classes.select}${hidden ? " hidden" : ""}`}>
      <InputLabel id={id + "-select-label"}>{label}</InputLabel>
      <MaterialSelect
        labelId={id + "-select-label"}
        id={id}
        value={value}
        onChange={(e) => onChange(e.target.value as string)}
      >
        {children}
      </MaterialSelect>
    </FormControl>
  )
}

function Checkbox({
  checked,
  onChange,
  label,
  hidden,
}: {
  checked: boolean
  onChange: (value: boolean) => void
  label: string
  hidden?: boolean
}) {
  return (
    <FormControlLabel
      className={`${hidden ? "hidden" : ""}`}
      control={
        <MaterialCheckbox
          checked={checked}
          onChange={(e: ChangeEvent<HTMLInputElement>) =>
            onChange(e.target.checked)
          }
        />
      }
      label={label}
    />
  )
}
