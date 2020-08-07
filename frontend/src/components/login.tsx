import React, { useState } from "react"
import { TextField, Button, FormHelperText } from "@material-ui/core"
import { createStyles, makeStyles, Theme } from "@material-ui/core/styles"
import { sendEmailPassword, Token } from "../lib/auth"

export default function Login({
  updateToken,
}: {
  updateToken: (cred: Token) => void
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
      <LoginForm updateToken={updateToken} />
    </div>
  )
}

function LoginForm({ updateToken }: { updateToken: (tok: Token) => void }) {
  const useStyles = makeStyles((theme: Theme) =>
    createStyles({
      loginForm: {
        display: "flex",
        "flex-direction": "column",
        "max-width": "50em",
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
  let [passwordError, setPasswordError] = useState(false)
  let [passwordMsg, setPasswordMsg] = useState("")
  let [password, setPassword] = useState("")
  let [buttonMsg, setButtonMsg] = useState("")
  function handleSubmit() {
    sendEmailPassword({ email: email, password: password })
      .then((tok) => updateToken(tok))
      .catch((e) => {
        if (e.message === "EmailNotFound") {
          setPasswordError(false)
          setPasswordMsg("")
          setEmailError(true)
          setEmailMsg("Email not found")
        } else if (e.message === "WrongPassword") {
          setEmailError(false)
          setEmailMsg("")
          setPasswordError(true)
          setPasswordMsg("Wrong password")
        } else {
          setButtonMsg(e.message)
        }
      })
  }
  return (
    <form className={classes.loginForm} data-testid="login-form">
      <TextField
        error={emailError}
        helperText={emailMsg}
        value={email}
        onChange={(e) => setEmail(e.target.value)}
        label="Email"
        type="email"
        autoComplete="email"
        inputProps={{ "data-testid": "email-input" }}
      />
      <TextField
        error={passwordError}
        helperText={passwordMsg}
        value={password}
        onChange={(e) => setPassword(e.target.value)}
        label="Password"
        type="password"
        autoComplete="current-password"
        inputProps={{ "data-testid": "password-input" }}
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
        data-testid="login-submit"
      >
        Submit
      </Button>
      <FormHelperText error={true} data-testid="login-button-msg">
        {buttonMsg}
      </FormHelperText>
    </form>
  )
}
