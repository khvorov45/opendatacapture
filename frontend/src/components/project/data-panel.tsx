import { makeStyles, Theme, createStyles } from "@material-ui/core"
import React, { useCallback, useEffect, useState } from "react"
import { trackPromise, usePromiseTracker } from "react-promise-tracker"
import { Redirect, Route, useParams, useRouteMatch } from "react-router-dom"
import { getAllTableNames } from "../../lib/api/project"
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
  const [tableNames, setTableNames] = useState<string[] | null>(null)
  const [errorMsg, setErrorMsg] = useState("")
  const { promiseInProgress } = usePromiseTracker({ area: "getAllTableNames" })
  const refreshTables = useCallback(() => {
    trackPromise(getAllTableNames(token, projectName), "getAllTableNames")
      .then((tables) => {
        setErrorMsg("")
        setTableNames(tables)
      })
      .catch((e) => setErrorMsg(e.message))
  }, [token, projectName])
  useEffect(() => {
    refreshTables()
  }, [refreshTables])
  const { url } = useRouteMatch()
  const classes = useStyles()
  return (
    <div data-testid="data-panel">
      <div className={classes.dataControl}>
        <ButtonArray errorMsg={errorMsg} errorTestId="refresh-tables-error">
          <RefreshButton
            onClick={refreshTables}
            inProgress={promiseInProgress}
            dataTestId="refresh-tables-button"
          />
        </ButtonArray>
        <SimpleNav links={tableNames ?? []} />
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
        <TableEntry />
      </Route>
    </div>
  )
}

function TableEntry() {
  const { tablename } = useParams<{ tablename: string }>()
  return <div>Table entry for {tablename}</div>
}
