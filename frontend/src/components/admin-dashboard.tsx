import React from "react"
import {
  CircularProgress,
  createStyles,
  makeStyles,
  Theme,
} from "@material-ui/core"

export default function AdminDashboard({ token }: { token: string | null }) {
  return (
    <div data-testid="admin-dashboard">
      {token ? <Main token={token} /> : <CircularProgress />}
    </div>
  )
}

function Main({ token }: { token: string }) {
  return <>Admin Dashboard</>
}
