import {
  makeStyles,
  Theme,
  createStyles,
  TableContainer,
  TableHead,
  TableBody,
  Table as MaterialTable,
  TextField,
  FormHelperText,
} from "@material-ui/core"
import React, { useCallback, useEffect, useMemo, useState } from "react"
import { trackPromise, usePromiseTracker } from "react-promise-tracker"
import {
  Redirect,
  Route,
  useLocation,
  useParams,
  useRouteMatch,
} from "react-router-dom"
import { useTable } from "react-table"
import { decodeUserTable } from "../../lib/api/io-validation"
import {
  ColMeta,
  getAllTableNames,
  getTableData,
  getTableMeta,
  insertData,
  removeAllTableData,
  TableData,
  TableMeta,
  TableRow,
} from "../../lib/api/project"
import {
  ButtonArray,
  CreateButton,
  RefreshButton,
  CheckButton,
  DeleteButton,
} from "../button"
import { SimpleNav } from "../nav"
import { StyledTableCell, StyledTableRow } from "../table"

const useStyles = makeStyles((theme: Theme) =>
  createStyles({
    dataControl: {
      display: "flex",
      borderBottom: `1px solid ${theme.palette.divider}`,
      "&>*:nth-child(odd)": {
        borderRight: `1px solid ${theme.palette.divider}`,
      },
      "& *": {
        height: 48,
      },
    },
    tableContainer: {
      width: "auto",
      "& table": {
        margin: "auto",
        width: "auto",
        "& td, & th": {
          textAlign: "center",
        },
      },
    },
  })
)

/**Fetches and displays table names, delegate error presentation to individual
 * tables though. Also let the table control panel refresh this list of table
 * names so that I only need one refresh button.
 */
export default function DataPanel({
  token,
  projectName,
}: {
  token: string
  projectName: string
}) {
  // Table list
  const [tableNames, setTableNames] = useState<string[] | null>(null)
  const [tableNamesError, setTableNamesError] = useState("")
  const refreshTables = useCallback(() => {
    trackPromise(getAllTableNames(token, projectName), "refresh")
      .then((tables) => {
        setTableNamesError("")
        setTableNames(tables)
      })
      .catch((e) => setTableNamesError(e.message))
  }, [token, projectName])
  useEffect(() => {
    refreshTables()
  }, [refreshTables])

  const { url } = useRouteMatch()
  const { pathname } = useLocation()
  return (
    <div data-testid="data-panel">
      <FormHelperText
        error={true}
        data-testid={"table-names-error"}
        className={tableNamesError === "" ? "nodisplay" : ""}
      >
        {tableNamesError}
      </FormHelperText>
      <SimpleNav
        links={tableNames ?? []}
        active={(l) => pathname.includes(`/project/${projectName}/data/${l}`)}
      />
      <Route exact path={url}>
        {tableNames === null ? (
          <></>
        ) : tableNames.length === 0 ? (
          "No tables found"
        ) : (
          <Redirect to={`${url}/${tableNames[0]}`} />
        )}
      </Route>
      <Route path={`${url}/:tablename`}>
        <TableEntry
          token={token}
          projectName={projectName}
          refreshTableLinks={refreshTables}
        />
      </Route>
    </div>
  )
}

/**This does all the single-table async data fetching/error handling */
function TableEntry({
  token,
  projectName,
  refreshTableLinks,
}: {
  token: string
  projectName: string
  refreshTableLinks: () => void
}) {
  const { tablename } = useParams<{ tablename: string }>()

  // Current meta
  const [meta, setMeta] = useState<TableMeta | null>(null)
  const [metaError, setMetaError] = useState("")
  const refreshMeta = useCallback(() => {
    trackPromise(getTableMeta(token, projectName, tablename), "refresh")
      .then((tm) => {
        setMetaError("")
        setMeta(tm)
      })
      .catch((e) => setMetaError(e.message))
  }, [token, projectName, tablename, setMetaError])

  // Current table data
  const [data, setData] = useState<TableData | null>(null)
  const [dataError, setDataError] = useState("")
  const refreshData = useCallback(() => {
    trackPromise(getTableData(token, projectName, tablename), "refresh")
      .then((td) => {
        setDataError("")
        setData(td)
      })
      .catch((e) => setDataError(e.message))
  }, [token, projectName, tablename, setDataError])
  // Refresh everything
  const refreshAll = useCallback(() => {
    refreshTableLinks()
    refreshMeta()
    refreshData()
  }, [refreshTableLinks, refreshMeta, refreshData])
  // Refresh only the table on load
  useEffect(() => {
    refreshMeta()
    refreshData()
  }, [refreshMeta, refreshData])

  // Delete all table data
  const [deleteError, setDeleteError] = useState("")
  const deleteAllData = useCallback(() => {
    trackPromise(removeAllTableData(token, projectName, tablename), "deleteAll")
      .then(() => {
        setDeleteError("")
        refreshData()
      })
      .catch((e) => setDeleteError(e.message))
  }, [token, projectName, tablename, setDeleteError, refreshData])

  // Promises
  const refreshPromise = usePromiseTracker({ area: "refresh" })
  const deleteAllPromise = usePromiseTracker({ area: "deleteAll" })

  return !meta || !data ? (
    <FormHelperText error={true}>{`${dataError}${metaError}`}</FormHelperText>
  ) : (
    <Table
      token={token}
      projectName={projectName}
      meta={meta}
      data={data}
      onNewRow={refreshData}
      onRefresh={refreshAll}
      refreshInProgress={refreshPromise.promiseInProgress}
      refreshError={`${dataError}${metaError}`}
      onDelete={deleteAllData}
      deleteError={deleteError}
      deleteInProgress={deleteAllPromise.promiseInProgress}
    />
  )
}

/**This does nothing async-related, just gets things and presents them */
function Table({
  token,
  projectName,
  meta,
  data,
  onNewRow,
  onRefresh,
  refreshInProgress,
  refreshError,
  onDelete,
  deleteInProgress,
  deleteError,
}: {
  token: string
  projectName: string
  meta: TableMeta
  data: TableData
  onNewRow: () => void
  onRefresh: () => void
  refreshInProgress: boolean
  refreshError: string
  onDelete: () => void
  deleteInProgress: boolean
  deleteError: string
}) {
  let decodedData: TableData = useMemo(() => {
    try {
      return decodeUserTable(meta, data)
    } catch (e) {
      // This happens when meta and data are out of sync. Since the database
      // guarantees that meta and data will eventually agree, do nothing and
      // wait
      return []
    }
  }, [meta, data])
  // New row form visibility
  const [newRow, setNewRow] = useState(data.length === 0)
  useEffect(() => {
    if (data.length === 0) {
      setNewRow(true)
    }
  }, [data])
  // Table stuff
  const columns = useMemo(
    () =>
      meta.cols.map((c) => {
        let accessor = (row: TableRow) => row[c.name]
        if (c.postgres_type === "boolean") {
          accessor = (row) => row[c.name].toString()
        } else if (c.postgres_type === "timestamp with time zone") {
          accessor = (row) => row[c.name].toISOString()
        }
        return { Header: c.name, accessor: accessor }
      }),
    [meta]
  )
  const {
    headers,
    rows,
    getTableProps,
    getTableBodyProps,
    prepareRow,
  } = useTable<TableRow>({
    columns: columns,
    data: decodedData,
  })
  const classes = useStyles()
  return (
    <TableContainer className={classes.tableContainer}>
      <MaterialTable {...getTableProps()}>
        <TableHead>
          <StyledTableRow data-testid="header-row">
            {/*Actual headers*/}
            {headers.map((header) => (
              <StyledTableCell {...header.getHeaderProps()}>
                {header.render("Header")}
              </StyledTableCell>
            ))}
            {/*Control buttons*/}
            <StyledTableCell>
              <ButtonArray errorMsg={`${refreshError}${deleteError}`}>
                <CreateButton
                  onClick={() => setNewRow((old) => !old)}
                  dataTestId="new-row-toggle"
                />
                <RefreshButton
                  onClick={onRefresh}
                  inProgress={refreshInProgress}
                  dataTestId="refresh-table-button"
                />
                <DeleteButton
                  onClick={onDelete}
                  inProgress={deleteInProgress}
                  dataTestId="delete-all-table-data-button"
                />
              </ButtonArray>
            </StyledTableCell>
          </StyledTableRow>
        </TableHead>
        <TableBody {...getTableBodyProps()}>
          <InputRow
            token={token}
            projectName={projectName}
            meta={meta}
            hidden={!newRow}
            onSubmit={onNewRow}
          />
          {rows.map((row) => {
            prepareRow(row)
            return (
              <StyledTableRow {...row.getRowProps()} data-testid="data-row">
                {row.cells.map((cell) => (
                  <StyledTableCell {...cell.getCellProps()}>
                    {cell.render("Cell")}
                  </StyledTableCell>
                ))}
                {/*Line up with control*/}
                <StyledTableCell />
              </StyledTableRow>
            )
          })}
        </TableBody>
      </MaterialTable>
    </TableContainer>
  )
}

function InputRow({
  token,
  projectName,
  meta,
  hidden,
  onSubmit,
}: {
  token: string
  projectName: string
  meta: TableMeta
  hidden: boolean
  onSubmit: () => void
}) {
  // To handle the duality of input (i.e. what I can sent to the DB vs what
  // the user types) I let each individual input field handle its own input
  // string which it attempts to validate (but not change) before sending it
  // here where it will be parsed into the row. Invalid strings are sent as
  // empty strings, so invalid input is empty input
  function convertValue(
    val: string,
    type: string
  ): string | number | boolean | Date {
    if (type === "integer") {
      return parseInt(val)
    }
    if (type === "real") {
      return parseFloat(val)
    }
    if (type === "boolean") {
      return val === "true"
    }
    if (type === "timestamp with time zone") {
      return new Date(val)
    }
    return val
  }
  const [parsedRecord, setParsedRecord] = useState<TableRow>({})
  function handleChange(col: ColMeta, val: string) {
    const newParsedRecord = { ...parsedRecord }
    if (val === "") {
      delete newParsedRecord[col.name]
    } else {
      newParsedRecord[col.name] = convertValue(val, col.postgres_type)
    }
    setParsedRecord(newParsedRecord)
  }
  const [errorMsg, setErrorMsg] = useState("")
  // Submit row
  const { promiseInProgress } = usePromiseTracker({ area: "submit-row" })
  function submitRow() {
    trackPromise(
      insertData(token, projectName, meta.name, [parsedRecord]),
      "submit-row"
    )
      .then(() => {
        setErrorMsg("")
        onSubmit()
      })
      .catch((e) => setErrorMsg(e.message))
  }
  return (
    <StyledTableRow
      className={hidden ? "nodisplay" : ""}
      data-testid="input-row"
    >
      {meta.cols.map((c) => (
        <StyledTableCell key={c.name}>
          <Input col={c} onChange={handleChange} />
        </StyledTableCell>
      ))}
      <StyledTableCell>
        <ButtonArray errorMsg={errorMsg}>
          <CheckButton
            dataTestId="submit-row-button"
            onClick={submitRow}
            inProgress={promiseInProgress}
          />
        </ButtonArray>
      </StyledTableCell>
    </StyledTableRow>
  )
}

function Input({
  col,
  onChange,
}: {
  col: ColMeta
  onChange: (col: ColMeta, val: string) => void
}) {
  // Not a fan of how lax JS number parsing is (e.g. '123ggg' parses as '123')
  function validate(val: string): boolean {
    if (val === "") {
      return true
    }
    if (col.postgres_type === "integer") {
      return !isNaN(parseInt(val))
    }
    if (col.postgres_type === "real") {
      return !isNaN(parseFloat(val))
    }
    if (col.postgres_type === "boolean") {
      return val === "true" || val === "false"
    }
    if (col.postgres_type === "timestamp with time zone") {
      return !isNaN(new Date(val).getTime())
    }
    return true
  }
  const [value, setValue] = useState("")
  const [valid, setValid] = useState(true)
  // Invalid value is equivalent to an empty entry
  function handleChange(val: string) {
    setValue(val)
    const valIsValid = validate(val)
    setValid(valIsValid)
    onChange(col, valIsValid ? val : "")
  }
  return (
    <TextField
      value={value}
      error={!valid}
      label={col.name}
      onChange={(e) => handleChange(e.target.value)}
    />
  )
}
