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
      backgroundColor: "var(--palette-bg-alt)",
      borderBottom: `1px solid ${theme.palette.divider}`,
    },
    tablePanel: {
      "&>*": {
        margin: "auto",
      },
    },
    tableControl: {
      borderBottom: `1px solid ${theme.palette.divider}`,
    },
    tableCards: {
      display: "flex",
      flexWrap: "wrap",
      justifyContent: "center",
      overflow: "auto",
    },
    tableCard: {
      display: "flex",
      flexDirection: "column",
      width: "350px",
      "&>.padded": {
        margin: "auto",
        paddingLeft: "10px",
        paddingRight: "10px",
        paddingBottom: "10px",
      },
      "&>.head": {
        paddingTop: 5,
      },
      "&>.cols": {
        flexGrow: 1,
      },
      border: `1px solid ${theme.palette.divider}`,
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
    <div className={classes.tablePanel}>
      <ButtonArray className={classes.tableControl} errorMsg={errorMsg} center>
        <CreateButton
          onClick={() => setRenderNew((old) => !old)}
          dataTestId="create-table-button"
        />
        <RefreshButton onClick={refreshTables} inProgress={promiseInProgress} />
      </ButtonArray>
      <NewTableForm
        token={token}
        projectName={projectName}
        onSubmit={refreshTables}
        tableSpec={tableSpec}
        noDisplay={!renderNew}
      />
      <TableCards tableSpec={tableSpec} />
    </div>
  )
}

function TableCards({ tableSpec }: { tableSpec: TableSpec }) {
  const classes = useStyles()
  return (
    <div className={classes.tableCards}>
      {tableSpec.map((tableMeta) => (
        <TableCard
          key={tableMeta.name}
          tableMeta={tableMeta}
          tableSpec={tableSpec}
        />
      ))}
    </div>
  )
}

function TableCard({
  tableMeta,
  tableSpec,
}: {
  tableMeta: TableMeta
  tableSpec: TableSpec
}) {
  const classes = useStyles()
  return (
    <div
      className={`${classes.tableCard}`}
      data-testid={`table-card-${tableMeta.name}`}
    >
      <div className="padded head">
        <TableHead
          inputTestId={`table-card-name-field-${tableMeta.name}`}
          value={tableMeta.name}
          disabled={true}
          onChange={(name) => {}}
        />
      </div>
      <NamedDivider name="Columns" />
      <div className="padded cols">
        {tableMeta.cols.map((c, i) => (
          <ColumnEntry
            key={i}
            tableSpec={tableSpec}
            readOnly={true}
            name={tableMeta.cols[i].name}
            onNameChange={(value) => {}}
            type={tableMeta.cols[i].postgres_type}
            onTypeChange={(value) => {}}
            primaryKey={tableMeta.cols[i].primary_key}
            onPKChange={(value) => {}}
            notNull={tableMeta.cols[i].not_null}
            onNNChange={(value) => {}}
            unique={tableMeta.cols[i].unique}
            onUniqueChange={(value) => {}}
            foreignKey={tableMeta.cols[i].foreign_key}
            onFKChange={(value) => {}}
            onRemove={() => {}}
          />
        ))}
      </div>
    </div>
  )
}

function TableHead({
  inputTestId,
  disabled,
  value,
  onChange,
}: {
  inputTestId: string
  disabled?: boolean
  value: string
  onChange: (s: string) => void
}) {
  return (
    <TextField
      inputProps={{
        "data-testid": inputTestId,
      }}
      label="Table name"
      value={value}
      disabled={disabled}
      onChange={(e) => onChange(e.target.value)}
    />
  )
}

function NewTableForm({
  token,
  projectName,
  onSubmit,
  tableSpec,
  noDisplay,
}: {
  token: string
  projectName: string
  onSubmit: () => void
  tableSpec: TableSpec
  noDisplay: boolean
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

  function removeCol(i: number) {
    const newCols = [...cols]
    newCols.splice(i, 1)
    setCols(newCols)
  }

  function handleClear() {
    setErrorMsg("")
    setName("")
    setCols([defaultCol])
  }

  const [errorMsg, setErrorMsg] = useState("")
  function handleSubmit() {
    const tableMeta = {
      name: name,
      cols: cols,
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
    <div
      className={`${classes.tableCard}${noDisplay ? " nodisplay" : ""}`}
      data-testid="new-table-form"
    >
      <div className="padded">
        <TableHead
          inputTestId="new-table-name-field"
          value={name}
          onChange={(newname) => setName(newname)}
        />
      </div>
      <NamedDivider name="Columns" />
      <div className="padded">
        {cols.map((c, i) => (
          <ColumnEntry
            key={i}
            tableSpec={tableSpec}
            name={cols[i].name}
            onNameChange={(value) => setColName(value, i)}
            type={cols[i].postgres_type}
            onTypeChange={(value) => setColType(value, i)}
            primaryKey={cols[i].primary_key}
            onPKChange={(value) => setColPK(value, i)}
            notNull={cols[i].not_null}
            onNNChange={(value) => setColNN(value, i)}
            unique={cols[i].unique}
            onUniqueChange={(value) => setColUnique(value, i)}
            foreignKey={cols[i].foreign_key}
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
  tableSpec,
  readOnly,
  name,
  onNameChange,
  type,
  onTypeChange,
  primaryKey,
  onPKChange,
  notNull,
  onNNChange,
  unique,
  onUniqueChange,
  foreignKey,
  onFKChange,
  onRemove,
}: {
  tableSpec: TableSpec
  readOnly?: boolean
  name: string
  onNameChange: (value: string) => void
  type: string
  onTypeChange: (value: string) => void
  primaryKey: boolean
  onPKChange: (value: boolean) => void
  notNull: boolean
  onNNChange: (value: boolean) => void
  unique: boolean
  onUniqueChange: (value: boolean) => void
  foreignKey: ForeignKey | null
  onFKChange: (value: ForeignKey | null) => void
  onRemove: () => void
}) {
  const allowedTypes = ["integer", "text"]

  const [foreignKeyCheckbox, setForeignKeyCheckbox] = useState(
    foreignKey !== null
  )
  function handleFKChange(newFK: boolean) {
    setForeignKeyCheckbox(newFK)
    if (!newFK) {
      onFKChange(null)
    }
  }
  const [foreignTable, setForeignTable] = useState(
    foreignKey ? foreignKey.table : ""
  )
  function handleForeignTableChange(newTable: string) {
    setForeignTable(newTable)
    // Auto-fill column if there is only one option
    let primaryKeys = tableSpec
      .find((t) => t.name === newTable)
      ?.cols.filter((c) => c.primary_key)
    let newColumn = foreignColumn
    if (primaryKeys?.length === 1) {
      newColumn = primaryKeys[0].name
      setForeignColumn(newColumn)
    }
    onFKChange({ table: newTable, column: newColumn })
  }
  const [foreignColumn, setForeignColumn] = useState(
    foreignKey ? foreignKey.column : ""
  )
  function handleForeignColumnChange(newCol: string) {
    setForeignColumn(newCol)
    onFKChange({ table: foreignTable, column: newCol })
  }

  const classes = useStyles()
  const theme = useTheme()
  return (
    <div className={classes.columnEntry}>
      <div>
        <TextField
          inputProps={{ "data-testid": "new-column-name-field" }}
          label="Name"
          value={name}
          onChange={(e) => {
            onNameChange(e.target.value)
          }}
          disabled={readOnly}
        />
        <Select
          id="type"
          value={type}
          onChange={onTypeChange}
          label="Type"
          readOnly={readOnly}
        >
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
          onChange={onPKChange}
          label="Primary key"
          readOnly={readOnly}
        />
        <Checkbox
          checked={notNull}
          onChange={onNNChange}
          label="Not null"
          hidden={primaryKey}
          readOnly={readOnly}
        />
        <Checkbox
          checked={unique}
          onChange={onUniqueChange}
          label="Unique"
          hidden={primaryKey}
          readOnly={readOnly}
        />
      </div>

      <div>
        <Checkbox
          checked={foreignKeyCheckbox}
          onChange={handleFKChange}
          label="Foreign key"
          readOnly={readOnly}
        />
        <Select
          id="fk-table"
          value={foreignTable}
          onChange={handleForeignTableChange}
          label="Table"
          hidden={!foreignKeyCheckbox}
          readOnly={readOnly}
        >
          {tableSpec
            .filter((t) => t.cols.some((c) => c.primary_key))
            .map((t) => (
              <MenuItem key={t.name} value={t.name}>
                {t.name}
              </MenuItem>
            ))}
        </Select>
        <Select
          id="fk-column"
          value={foreignColumn}
          onChange={handleForeignColumnChange}
          label="Column"
          hidden={!foreignKeyCheckbox}
          readOnly={readOnly}
        >
          {tableSpec
            .find((t) => t.name === foreignTable)
            ?.cols.filter((c) => c.primary_key)
            .map((c) => (
              <MenuItem key={c.name} value={c.name}>
                {c.name}
              </MenuItem>
            ))}
        </Select>
      </div>
      <div className={`delete${readOnly ? " nodisplay" : ""}`}>
        <IconButton onClick={(e) => onRemove()}>
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
  readOnly,
}: {
  children: ReactNode
  value: string
  onChange: (value: string) => void
  id: string
  label: string
  hidden?: boolean
  readOnly?: boolean
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
        disabled={readOnly}
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
  readOnly,
}: {
  checked: boolean
  onChange: (value: boolean) => void
  label: string
  hidden?: boolean
  readOnly?: boolean
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
          disabled={readOnly}
        />
      }
      label={label}
    />
  )
}
