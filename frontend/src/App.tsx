import React, { useState } from "react"
import { createMuiTheme, ThemeProvider, Theme } from "@material-ui/core/styles"
import Nav from "./components/nav"
import Login from "./components/login"
import { Token } from "./lib/auth"
import CssBaseline from "@material-ui/core/CssBaseline"
import { BrowserRouter as Router, Switch, Route } from "react-router-dom"
import Home from "./components/home"

function createThemeFromPalette(palette: "dark" | "light"): Theme {
  return createMuiTheme({
    palette: {
      type: palette,
    },
  })
}

export default function App({
  initPalette,
}: {
  initPalette: "dark" | "light"
}) {
  const [darkState, setDarkState] = useState(initPalette === "dark")
  const [theme, setTheme] = useState(createThemeFromPalette(initPalette))
  const handleThemeChange = () => {
    let newDarkState = !darkState
    setDarkState(newDarkState)
    let newPalette: "dark" | "light" = newDarkState ? "dark" : "light"
    setTheme(createThemeFromPalette(newPalette))
    document.documentElement.setAttribute("theme", newPalette)
    localStorage.setItem("theme", newPalette)
  }
  function updateToken(tok: Token) {
    console.log("Login success: " + JSON.stringify(tok))
  }
  function updateTokenError(msg: string) {
    console.log("login failed: " + msg)
  }
  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Nav handleThemeChange={handleThemeChange} />
      <Router>
        <Switch>
          <Route path="/login">
            <Login
              updateToken={updateToken}
              updateTokenError={updateTokenError}
            />
          </Route>
          <Route path="/">
            <Home />
          </Route>
        </Switch>
      </Router>
    </ThemeProvider>
  )
}
