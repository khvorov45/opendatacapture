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
  TableData,
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
  useEffect(() => {
    refreshTables()
  }, [refreshTables])

  // Current table
  const { pathname } = useLocation()
  const currentTable =
    pathname.match(/^\/project\/[^/]*\/data\/([^/]*)/)?.[1] ?? null

  // Current table data
  const [data, setData] = useState<TableData | null>(null)
  const refreshData = useCallback(() => {
    if (!currentTable) {
      return
    }
    trackPromise(getTableData(token, projectName, currentTable), "refresh")
      .then((td) => {
        setErrorMsg("")
        setData(td)
      })
      .catch((e) => setErrorMsg(e.message))
  }, [token, projectName, currentTable])
  useEffect(() => {
    refreshData()
  }, [refreshData])

  const { url } = useRouteMatch()
  const classes = useStyles()
  return (
    <div data-testid="data-panel">
      <div className={classes.dataControl}>
        <ButtonArray errorMsg={errorMsg} errorTestId="refresh-tables-error">
          <RefreshButton
            onClick={() => {
              refreshTables()
              refreshData()
            }}
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
        <TableEntry data={data ?? []} />
      </Route>
    </div>
  )
}

function TableEntry({ data }: { data: TableData }) {
  const { tablename } = useParams<{ tablename: string }>()
  return (
    <div>
      Table entry for {tablename} with data {JSON.stringify(data)}
    </div>
  )
}
