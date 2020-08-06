import React from "react"
import { TextField } from "@material-ui/core"
import { createStyles, makeStyles, Theme } from "@material-ui/core/styles"

const useStyles = makeStyles((theme: Theme) =>
  createStyles({
    loginPage: {
      display: "flex",
      justifyContent: "center",
    },
    loginForm: {
      display: "flex",
      "flex-direction": "column",
      "max-width": "50ch",
    },
  })
)

export default function Login() {
  const classes = useStyles()
  return (
    <div className={classes.loginPage}>
      <div className={classes.loginForm}>
        <TextField label="Email" type="email" />
        <TextField label="Password" type="password" />
      </div>
    </div>
  )
}

function LoginForm() {
  return (
    <>
      <TextField label="Email" type="email" />
      <TextField label="Password" type="password" />
    </>
  )
}
