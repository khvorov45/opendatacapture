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
  // New row form visibility
  const [newRow, setNewRow] = useState(data.length === 0)
  useEffect(() => {
    if (data.length === 0) {
      setNewRow(true)
    }
  }, [data])
  // Table stuff
  const columns = useMemo(
    () => meta.cols.map((c) => ({ Header: c.name, accessor: c.name })),
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
    data: data,
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
              <StyledTableRow {...row.getRowProps()}>
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
  const [record, setRecord] = useState<TableRow>({})
  function handleChange(fieldName: string, val: number | string) {
    const newRecord = { ...record }
    if (val === "") {
      delete newRecord[fieldName]
    } else {
      newRecord[fieldName] = val
    }
    setRecord(newRecord)
  }
  const [errorMsg, setErrorMsg] = useState("")
  // Submit row
  function submitRow() {
    trackPromise(
      insertData(token, projectName, meta.name, [record]),
      "sumbit-row"
    )
      .then(() => {
        setErrorMsg("")
        setRecord({})
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
          <Input col={c} onChange={handleChange} val={record[c.name] ?? ""} />
        </StyledTableCell>
      ))}
      <StyledTableCell>
        <ButtonArray errorMsg={errorMsg}>
          <CheckButton dataTestId="submit-row-button" onClick={submitRow} />
        </ButtonArray>
      </StyledTableCell>
    </StyledTableRow>
  )
}

function Input({
  col,
  onChange,
  val,
}: {
  col: ColMeta
  onChange: (fieldName: string, val: number | string) => void
  val: string | number
}) {
  function convertValue(val: string): string | number {
    if (col.postgres_type === "integer") {
      return parseInt(val)
    }
    return val
  }
  const [error, setError] = useState(false)
  function handleChange(val: string) {
    // Deleted everything
    if (val === "") {
      setError(false)
      onChange(col.name, val)
      return
    }
    // Some input, need to convert
    const convertedVal = convertValue(val)
    // Conversion errors - don't notify parent
    if (typeof convertedVal === "number" && isNaN(convertedVal)) {
      setError(true)
      return
    }
    // Successfull conversion
    setError(false)
    onChange(col.name, convertedVal)
  }
  return (
    <TextField
      value={val}
      error={error}
      label={col.name}
      onChange={(e) => handleChange(e.target.value)}
    />
  )
}
