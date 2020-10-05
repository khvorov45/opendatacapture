import { makeStyles, Theme, createStyles } from "@material-ui/core"
import React, { useCallback, useEffect, useState } from "react"
import { trackPromise, usePromiseTracker } from "react-promise-tracker"
import {
  Redirect,
  Route,
  useLocation,
  useParams,
  useRouteMatch,
} from "react-router-dom"
import {
  getAllTableNames,
  getTableData,
  getTableMeta,
  TableData,
  TableMeta,
} from "../../lib/api/project"
import { ButtonArray, RefreshButton } from "../button"
import { SimpleNav } from "../nav"

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
  const [errorMsg, setErrorMsg] = useState("")
  const { promiseInProgress } = usePromiseTracker({ area: "refresh" })
  const refreshTables = useCallback(() => {
    trackPromise(getAllTableNames(token, projectName), "refresh")
      .then((tables) => {
        setErrorMsg("")
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
          setErrorMsg={setErrorMsg}
        />
      </Route>
    </div>
  )
}

function TableEntry({
  token,
  projectName,
  refresh,
  setErrorMsg,
}: {
  token: string
  projectName: string
  refresh: number
  setErrorMsg: (s: string) => void
}) {
  const { tablename } = useParams<{ tablename: string }>()
  // Current meta
  const [meta, setMeta] = useState<TableMeta | null>(null)
  const refreshMeta = useCallback(() => {
    trackPromise(getTableMeta(token, projectName, tablename), "refresh")
      .then((tm) => {
        setErrorMsg("")
        setMeta(tm)
      })
      .catch((e) => setErrorMsg(e.message))
  }, [token, projectName, tablename, setErrorMsg])

  // Current table data
  const [data, setData] = useState<TableData | null>(null)
  const refreshData = useCallback(() => {
    trackPromise(getTableData(token, projectName, tablename), "refresh")
      .then((td) => {
        setErrorMsg("")
        setData(td)
      })
      .catch((e) => setErrorMsg(e.message))
  }, [token, projectName, tablename, setErrorMsg])

  // Refresh everything
  const refreshAll = useCallback(() => {
    refreshMeta()
    refreshData()
  }, [refreshMeta, refreshData])
  useEffect(() => {
    refreshAll()
  }, [refreshAll, refresh])

  return (
    <div>
      Table entry for {tablename} with meta {JSON.stringify(meta)} and data{" "}
      {JSON.stringify(data)}
    </div>
  )
}
