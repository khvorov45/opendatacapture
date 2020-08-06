import React, { useState } from "react"
import { TextField, Button } from "@material-ui/core"
import { createStyles, makeStyles, Theme } from "@material-ui/core/styles"
import { sendEmailPassword, IdToken } from "../lib/auth"

export default function Login({
  updateCred,
}: {
  updateCred: (cred: IdToken) => void
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
      <LoginForm updateCred={updateCred} />
    </div>
  )
}

function LoginForm({ updateCred }: { updateCred: (cred: IdToken) => void }) {
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
  let [password, setPassword] = useState("")
  function handleSubmit() {
    sendEmailPassword({ email: email, password: password })
      .then((cred) => console.log("Got cred: " + cred))
      .catch((e) => console.error("could not get credentials: " + e))
  }
  return (
    <div className={classes.loginForm}>
      <TextField
        value={email}
        onChange={(e) => setEmail(e.target.value)}
        label="Email"
        type="email"
      />
      <TextField
        value={password}
        onChange={(e) => setPassword(e.target.value)}
        label="Password"
        type="password"
      />
      <Button
        variant="contained"
        color="primary"
        className={classes.submitButton}
        onClick={(e) => handleSubmit()}
      >
        Submit
      </Button>
    </div>
  )
}
