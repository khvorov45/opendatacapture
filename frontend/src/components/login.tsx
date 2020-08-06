import React, { useState } from "react"
import { TextField, Button } from "@material-ui/core"
import { createStyles, makeStyles, Theme } from "@material-ui/core/styles"
import { sendEmailPassword, Token } from "../lib/auth"

export default function Login({
  updateToken,
  updateTokenError,
}: {
  updateToken: (cred: Token) => void
  updateTokenError: (msg: string) => void
}) {
  const useStyles = makeStyles((theme: Theme) =>
    createStyles({
      loginPage: {
        display: "flex",
        justifyContent: "center",
      },
    })
  )
  const classes = useStyles()
  return (
    <div className={classes.loginPage}>
      <LoginForm
        updateToken={updateToken}
        updateTokenError={updateTokenError}
      />
    </div>
  )
}

function LoginForm({
  updateToken,
  updateTokenError,
}: {
  updateToken: (cred: Token) => void
  updateTokenError: (msg: string) => void
}) {
  const useStyles = makeStyles((theme: Theme) =>
    createStyles({
      loginForm: {
        display: "flex",
        "flex-direction": "column",
        "max-width": "50ch",
      },
      submitButton: {
        "margin-top": "2em",
      },
    })
  )
  const classes = useStyles()
  let [email, setEmail] = useState("")
  let [emailError, setEmailError] = useState(false)
  let [emailMsg, setEmailMsg] = useState("")
  let [password, setPassword] = useState("")
  function handleSubmit() {
    sendEmailPassword({ email: email, password: password })
      .then((tok) => updateToken(tok))
      .catch((e) => {
        if (e.message === "EmailNotFound") {
          setEmailError(true)
          setEmailMsg("Email not found")
        } else {
          updateTokenError(e.message)
        }
      })
  }
  return (
    <form className={classes.loginForm}>
      <TextField
        error={emailError}
        helperText={emailMsg}
        value={email}
        onChange={(e) => setEmail(e.target.value)}
        label="Email"
        type="email"
        autoComplete="email"
      />
      <TextField
        value={password}
        onChange={(e) => setPassword(e.target.value)}
        label="Password"
        type="password"
        autoComplete="current-password"
      />
      <Button
        variant="contained"
        color="primary"
        type="submit"
        className={classes.submitButton}
        onClick={(e) => {
          e.preventDefault()
          handleSubmit()
        }}
      >
        Submit
      </Button>
    </form>
  )
}
