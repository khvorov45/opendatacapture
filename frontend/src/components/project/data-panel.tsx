import {
  makeStyles,
  Theme,
  createStyles,
  TableContainer,
  TableHead,
  TableBody,
  Table as MaterialTable,
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
  getAllTableNames,
  getTableData,
  getTableMeta,
  TableData,
  TableMeta,
  TableRow,
} from "../../lib/api/project"
import { ButtonArray, RefreshButton } from "../button"
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
  })
)

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
  const { promiseInProgress } = usePromiseTracker({ area: "refresh" })
  const refreshTables = useCallback(() => {
    trackPromise(getAllTableNames(token, projectName), "refresh")
      .then((tables) => {
        setTableNamesError("")
        setTableNames(tables)
      })
      .catch((e) => setErrorMsg(e.message))
  }, [token, projectName])

  // Let the table know it needs to refresh
  const [refreshCounter, setRefreshCounter] = useState(0)

  // Refresh everything
  const refreshAll = useCallback(() => {
    refreshTables()
    setRefreshCounter((old) => old + 1)
  }, [refreshTables, setRefreshCounter])
  useEffect(() => {
    refreshAll()
  }, [refreshAll])

  // Update error
  const [errorMsg, setErrorMsg] = useState("")
  function updateErrorMsg(msg: string) {
    setErrorMsg(tableNamesError + msg)
  }

  const { url } = useRouteMatch()
  const { pathname } = useLocation()
  const classes = useStyles()
  return (
    <div data-testid="data-panel">
      <div className={classes.dataControl}>
        <ButtonArray errorMsg={errorMsg} errorTestId="refresh-tables-error">
          <RefreshButton
            onClick={refreshAll}
            inProgress={promiseInProgress}
            dataTestId="refresh-tables-button"
          />
        </ButtonArray>
        <SimpleNav
          links={tableNames ?? []}
          active={(l) => pathname.includes(`/project/${projectName}/data/${l}`)}
        />
      </div>
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
          refresh={refreshCounter}
          updateErrorMsg={updateErrorMsg}
        />
      </Route>
    </div>
  )
}

function TableEntry({
  token,
  projectName,
  refresh,
  updateErrorMsg,
}: {
  token: string
  projectName: string
  refresh: number
  updateErrorMsg: (s: string) => void
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
    refreshMeta()
    refreshData()
  }, [refreshMeta, refreshData])
  useEffect(() => {
    refreshAll()
  }, [refreshAll, refresh])

  // Update errors
  useEffect(() => {
    updateErrorMsg(metaError + dataError)
  }, [updateErrorMsg, metaError, dataError])

  return !meta || !data ? <></> : <Table meta={meta} data={data} />
}

function Table({ meta, data }: { meta: TableMeta; data: TableData }) {
  const columns = useMemo(
    () => meta.cols.map((c) => ({ Header: c.name, accessor: c.name })),
    [meta]
  )
  const { headers, rows, getTableProps, getTableBodyProps } = useTable<
    TableRow
  >({
    columns: columns,
    data: data,
  })
  return (
    <TableContainer>
      <MaterialTable {...getTableProps()}>
        <TableHead>
          <StyledTableRow>
            {headers.map((header) => (
              <StyledTableCell {...header.getHeaderProps()}>
                {header.render("Header")}
              </StyledTableCell>
            ))}
          </StyledTableRow>
        </TableHead>
        <TableBody {...getTableBodyProps()}>
          {rows.map((row) => (
            <StyledTableRow {...row.getRowProps()}>
              {row.cells.map((cell) => (
                <StyledTableCell {...cell.getCellProps()}>
                  {cell.render("Cell")}
                </StyledTableCell>
              ))}
            </StyledTableRow>
          ))}
        </TableBody>
      </MaterialTable>
    </TableContainer>
  )
}
