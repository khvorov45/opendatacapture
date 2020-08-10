import React, { useState } from "react"
import { createMuiTheme, ThemeProvider, Theme } from "@material-ui/core/styles"
import Nav from "./components/nav"
import Login from "./components/login"
import { tokenFetcher, tokenValidator } from "./lib/auth"
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
  initToken,
}: {
  initPalette: "dark" | "light"
  initToken: string | null
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
  const [token, setToken] = useState<string | null>(initToken)
  function updateToken(tok: string) {
    setToken(tok)
    localStorage.setItem("token", tok)
  }
  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Nav handleThemeChange={handleThemeChange} />
      <Router>
        <Switch>
          <Route path="/login">
            <Login updateToken={updateToken} tokenFetcher={tokenFetcher} />
          </Route>
          <Route path="/">
            <Home token={token} tokenValidator={tokenValidator} />
          </Route>
        </Switch>
      </Router>
    </ThemeProvider>
  )
}
