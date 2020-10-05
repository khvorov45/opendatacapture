import React, { useCallback, useEffect, useState } from "react"
import { trackPromise } from "react-promise-tracker"
import { getAllTableNames } from "../../lib/api/project"

export default function DataPanel({
  token,
  projectName,
}: {
  token: string
  projectName: string
}) {
  const [tableNames, setTableNames] = useState<string[] | null>(null)
  const [errorMsg, setErrorMsg] = useState("")
  const refreshTables = useCallback(() => {
    trackPromise(getAllTableNames(token, projectName), "getAllMeta")
      .then((tables) => {
        setErrorMsg("")
        setTableNames(tables)
      })
      .catch((e) => setErrorMsg(e.message))
  }, [token, projectName])
  useEffect(() => {
    refreshTables()
  }, [refreshTables])
  return (
    <div data-testid="data-panel">
      {tableNames === null ? (
        <></>
      ) : tableNames.length === 0 ? (
        "No tables found"
      ) : (
        <Main token={token} projectName={projectName} tableNames={tableNames} />
      )}
    </div>
  )
}

function Main({
  token,
  projectName,
  tableNames,
}: {
  token: string
  projectName: string
  tableNames: string[]
}) {
  console.log(tableNames)
  return <div data-testid="data-main">Main data here</div>
}
