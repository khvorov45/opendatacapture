import {
  createStyles,
  IconButton,
  MenuItem,
  TextField,
  Theme,
  useTheme,
} from "@material-ui/core"
import makeStyles from "@material-ui/core/styles/makeStyles"
import Check from "@material-ui/icons/Check"
import Clear from "@material-ui/icons/Clear"
import Remove from "@material-ui/icons/Remove"
import Edit from "@material-ui/icons/Edit"
import React, { useState, useEffect, useCallback } from "react"
import { usePromiseTracker, trackPromise } from "react-promise-tracker"
import {
  TableSpec,
  getAllMeta,
  TableMeta,
  removeTable,
  ColMeta,
  ForeignKey,
  createTable,
} from "../../lib/api/project"
import {
  ButtonArray,
  CreateButton,
  RefreshButton,
  DeleteButton,
  IconButtonWithProgress,
} from "../button"
import { NamedDivider } from "../divider"
import Select from "../select"
import Checkbox from "../checkbox"

const useStyles = makeStyles((theme: Theme) =>
  createStyles({
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
        display: "flex",
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
  })
)

export default function TablePanel({
  token,
  projectName,
}: {
  token: string
  projectName: string
}) {
  let [renderNew, setRenderNew] = useState(false)
  let [tableSpec, setTableSpec] = useState<TableSpec | null>(null)
  let [errorMsg, setErrorMsg] = useState("")

  useEffect(() => {
    if (tableSpec?.length === 0) {
      setRenderNew(true)
    }
  }, [tableSpec])

  const { promiseInProgress } = usePromiseTracker({ area: "getAllMeta" })
  const refreshTables = useCallback(() => {
    trackPromise(getAllMeta(token, projectName), "getAllMeta")
      .then((tables) => {
        setErrorMsg("")
        setTableSpec(tables)
      })
      .catch((e) => setErrorMsg(e.message))
  }, [token, projectName])

  useEffect(() => {
    refreshTables()
  }, [refreshTables])

  const classes = useStyles()
  return (
    <div className={classes.tablePanel} data-testid="table-panel">
      <ButtonArray
        className={classes.tableControl}
        errorMsg={errorMsg}
        errorTestId="refresh-tables-error"
        center
      >
        <CreateButton
          onClick={() => setRenderNew((old) => !old)}
          dataTestId="create-table-button"
        />
        <RefreshButton
          onClick={refreshTables}
          inProgress={promiseInProgress}
          dataTestId="refresh-tables-button"
        />
      </ButtonArray>
      <NewTableForm
        token={token}
        projectName={projectName}
        onSubmit={refreshTables}
        tableSpec={tableSpec ?? []}
        noDisplay={!renderNew}
      />
      <TableCards
        tableSpec={tableSpec ?? []}
        token={token}
        projectName={projectName}
        onDelete={refreshTables}
      />
    </div>
  )
}

function TableCards({
  token,
  projectName,
  tableSpec,
  onDelete,
}: {
  token: string
  projectName: string
  tableSpec: TableSpec
  onDelete: () => void
}) {
  const classes = useStyles()
  return (
    <div className={classes.tableCards}>
      {tableSpec.map((tableMeta) => (
        <TableCard
          key={tableMeta.name}
          token={token}
          projectName={projectName}
          tableMeta={tableMeta}
          tableSpec={tableSpec}
          onDelete={onDelete}
        />
      ))}
    </div>
  )
}

function TableCard({
  token,
  projectName,
  tableMeta,
  tableSpec,
  onDelete,
}: {
  token: string
  projectName: string
  tableMeta: TableMeta
  tableSpec: TableSpec
  onDelete: () => void
}) {
  const classes = useStyles()
  const [editable, setEditable] = useState(false)
  const [errorMsg, setErrorMsg] = useState("")
  const [deleted, setDeleted] = useState(false)
  const { promiseInProgress } = usePromiseTracker({ area: "delete-table" })
  function handleDelete() {
    trackPromise(
      removeTable(token, projectName, tableMeta.name),
      "delete-table"
    )
      .then(() => {
        setDeleted(true)
        setErrorMsg("")
        onDelete()
      })
      .catch((e) => {
        setErrorMsg(e.message)
      })
  }
  return (
    <div
      className={`${classes.tableCard}${deleted ? " nodisplay" : ""}`}
      data-testid={`table-card-${tableMeta.name}`}
    >
      <div className="padded head">
        <TableHead
          inputTestId={`table-card-name-field`}
          value={tableMeta.name}
          disabled={!editable}
          onChange={(name) => {}}
        />
        <ButtonArray errorMsg={errorMsg} errorTestId="delete-table-error">
          <IconButton
            onClick={(e) => setEditable((old) => !old)}
            data-testid="enable-edit"
          >
            <Edit />
          </IconButton>
          <DeleteButton
            onClick={handleDelete}
            dataTestId="delete-table-button"
            inProgress={promiseInProgress}
          />
        </ButtonArray>
      </div>
      <NamedDivider name="Columns" />
      <div className="padded cols">
        {tableMeta.cols.map((c, i) => (
          <ColumnEntry
            key={i}
            tableSpec={tableSpec}
            readOnly={!editable}
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
            dataTestId={`table-${tableMeta.name}-column-entry-${i}`}
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
    if (newCols.length === 0) {
      setCols([defaultCol])
    } else {
      setCols(newCols)
    }
  }

  function handleClear() {
    setErrorMsg("")
    setName("")
    setCols([defaultCol])
  }

  const isViable = useCallback(() => {
    // Table name
    if (name === "") {
      return false
    }
    // Column with no name or type
    const badCol = cols.find((c) => c.name === "" || c.postgres_type === "")
    if (badCol) {
      return false
    }
    return true
  }, [cols, name])

  const [errorMsg, setErrorMsg] = useState("")
  const { promiseInProgress } = usePromiseTracker({ area: "submit-table" })
  function handleSubmit() {
    const tableMeta = {
      name: name,
      cols: cols,
    }
    trackPromise(createTable(token, projectName, tableMeta), "submit-table")
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
            dataTestId={`new-column-entry-${i}`}
          />
        ))}
      </div>
      <div>
        <CreateButton dataTestId={"add-column"} onClick={addCol} />
      </div>
      <NamedDivider name="" />
      <ButtonArray
        center
        className={"buttons"}
        errorMsg={errorMsg}
        errorTestId="table-submit-error"
      >
        <IconButtonWithProgress
          onClick={handleSubmit}
          dataTestId="submit-table-button"
          disabled={!isViable()}
          inProgress={promiseInProgress}
        >
          <Check
            htmlColor={
              isViable()
                ? theme.palette.success.main
                : theme.palette.text.disabled
            }
          />
        </IconButtonWithProgress>
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
  dataTestId,
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
  dataTestId: string
}) {
  const allowedTypes = ["integer", "text"]

  // FK checkbox
  function handleFKChange(newFK: boolean) {
    // Make sure the FK is always viable
    if (newFK) {
      handleForeignTableChange(availableForeignTables()[0].name)
    } else {
      onFKChange(null)
    }
  }

  // Foreign key table/column selection
  const availableForeignTables = useCallback(() => {
    // Single-column foreign key can only refer to a single-column primary key
    return tableSpec.filter(
      (t) => t.cols.filter((c) => c.primary_key).length === 1
    )
  }, [tableSpec])
  function handleForeignTableChange(newTable: string) {
    // There is only one column option per available table with my constraints
    const newForeignColumn = tableSpec
      .find((t) => t.name === newTable)
      ?.cols.find((c) => c.primary_key)
    // We can't possibly not find a column considering the constraints on
    // available foreign tables
    /* istanbul ignore next */
    if (newForeignColumn) {
      onFKChange({ table: newTable, column: newForeignColumn.name })
    }
  }
  const foreingColumn = foreignKey ? foreignKey.column : ""

  const classes = useStyles()
  const theme = useTheme()
  return (
    <div className={classes.columnEntry} data-testid={dataTestId}>
      <div>
        <TextField
          inputProps={{
            "data-testid": "new-column-name-field",
          }}
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
          dataTestId={"new-column-type-select"}
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
          dataTestId={"primary-key"}
        />
        <Checkbox
          checked={notNull}
          onChange={onNNChange}
          label="Not null"
          hidden={primaryKey}
          readOnly={readOnly}
          dataTestId={"not-null"}
        />
        <Checkbox
          checked={unique}
          onChange={onUniqueChange}
          label="Unique"
          hidden={primaryKey}
          readOnly={readOnly}
          dataTestId={"unique"}
        />
      </div>

      <div>
        <Checkbox
          checked={foreignKey !== null}
          onChange={handleFKChange}
          label="Foreign key"
          readOnly={readOnly || availableForeignTables().length === 0}
          dataTestId={"foreign-key"}
        />
        <Select
          id="fk-table"
          value={foreignKey ? foreignKey.table : ""}
          onChange={handleForeignTableChange}
          label="Table"
          hidden={foreignKey === null}
          readOnly={readOnly || availableForeignTables().length === 0}
          dataTestId={"foreign-table-select"}
        >
          {availableForeignTables().map((t) => (
            <MenuItem key={t.name} value={t.name}>
              {t.name}
            </MenuItem>
          ))}
        </Select>
        <Select
          id="fk-column"
          value={foreingColumn}
          label="Column"
          hidden={foreignKey === null}
          readOnly={true}
          dataTestId={"foreign-column-select"}
        >
          <MenuItem value={foreingColumn}>{foreingColumn}</MenuItem>
        </Select>
      </div>
      <div className={`delete${readOnly ? " hidden" : ""}`}>
        <IconButton onClick={(e) => onRemove()} data-testid="remove-column">
          <Remove htmlColor={theme.palette.error.main} />
        </IconButton>
      </div>
    </div>
  )
}
