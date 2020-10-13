import React, { ReactNode, useState } from "react"
import { createMuiTheme, ThemeProvider, Theme } from "@material-ui/core/styles"
import Nav from "./components/nav"
import Login from "./components/login"
import Project from "./components/project/project"
import { Token, tokenValidator } from "./lib/api/auth"
import CssBaseline from "@material-ui/core/CssBaseline"
import {
  BrowserRouter as Router,
  Switch,
  Route,
  Redirect,
} from "react-router-dom"
import Home from "./components/home"
import { AuthStatus, useToken } from "./lib/hooks"

function createThemeFromPalette(palette: "dark" | "light"): Theme {
  return createMuiTheme({
    palette: {
      type: palette,
      background: {
        paper: "var(--palette-bg)",
        default: "var(--palette-bg)",
      },
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
  // Theme
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
  // Token
  const [token, setToken] = useState<string | null>(initToken)
  const [lastRefresh, setLastRefresh] = useState(
    new Date(localStorage.getItem("last-refresh") ?? 0)
  )
  function updateToken(tok: Token) {
    setToken(tok.token)
    localStorage.setItem("token", tok.token)
    setLastRefresh(tok.created)
    localStorage.setItem("last-refresh", tok.created.toISOString())
  }
  const { auth } = useToken(token, tokenValidator)
  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Router>
        <Nav handleThemeChange={handleThemeChange} />
        <Switch>
          <Route exact path="/login">
            {auth === AuthStatus.Ok ? (
              <Redirect to="/" />
            ) : (
              <Login updateToken={updateToken} />
            )}
          </Route>
          <AuthRoute exact path="/" auth={auth}>
            <Home token={token} />
          </AuthRoute>
          <AuthRoute path="/project/:name" auth={auth}>
            <Project token={token} />
          </AuthRoute>
        </Switch>
      </Router>
    </ThemeProvider>
  )
}

function AuthRoute({
  path,
  auth,
  children,
  exact,
}: {
  path: string
  auth: AuthStatus
  children: ReactNode
  exact?: boolean
}) {
  return (
    <Route exact={exact} path={path}>
      {auth === AuthStatus.Err ? (
        <Redirect to="/login" />
      ) : auth === AuthStatus.InProgress ? (
        <></>
      ) : (
        children
      )}
    </Route>
  )
}
