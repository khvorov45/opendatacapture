import React, { ReactNode, useEffect, useState } from "react"
import { createMuiTheme, ThemeProvider, Theme } from "@material-ui/core/styles"
import Nav from "./components/nav"
import Login from "./components/login"
import Project from "./components/project/project"
import {
  Access,
  refreshToken,
  removeToken,
  Token,
  tokenValidator,
  User,
} from "./lib/api/auth"
import CssBaseline from "@material-ui/core/CssBaseline"
import {
  BrowserRouter as Router,
  Switch,
  Route,
  Redirect,
} from "react-router-dom"
import Home from "./components/home"
import { AuthStatus, useToken } from "./lib/hooks"
import { themeInit } from "./lib/theme"
import { TOKEN_HOURS_TO_REFRESH } from "./lib/config"
import AdminDashboard from "./components/admin-dashboard"

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

export default function App() {
  // Theme
  const palette = themeInit()
  const [darkState, setDarkState] = useState(palette === "dark")
  const [theme, setTheme] = useState(createThemeFromPalette(palette))
  const handleThemeChange = () => {
    let newDarkState = !darkState
    setDarkState(newDarkState)
    let newPalette: "dark" | "light" = newDarkState ? "dark" : "light"
    setTheme(createThemeFromPalette(newPalette))
    document.documentElement.setAttribute("theme", newPalette)
    localStorage.setItem("theme", newPalette)
  }
  // Token
  const [token, setToken] = useState<string | null>(
    localStorage.getItem("token")
  )
  const [lastRefresh, setLastRefresh] = useState(
    new Date(localStorage.getItem("last-refresh") ?? 0)
  )
  function updateToken(tok: Token) {
    setLastRefresh(tok.created)
    localStorage.setItem("last-refresh", tok.created.toISOString())
    setToken(tok.token)
    localStorage.setItem("token", tok.token)
  }
  function handleLogout() {
    const old_token = token
    localStorage.removeItem("last-refresh")
    localStorage.removeItem("token")
    setLastRefresh(new Date(0))
    setToken(null)
    if (old_token) {
      removeToken(old_token).catch((e) => console.error(e.message))
    }
  }
  const { user, auth } = useToken(token, tokenValidator)
  useEffect(() => {
    function conditionalRefresh() {
      // Gotta wait until we actually get a good token from somewhere
      if (auth !== AuthStatus.Ok || !token) {
        return
      }
      if (
        new Date().getTime() - lastRefresh.getTime() >
        TOKEN_HOURS_TO_REFRESH * 60 * 60 * 1000
      ) {
        refreshToken(token)
          .then(updateToken)
          .catch((e) => console.error(e.message))
      }
    }
    conditionalRefresh()
    const interval = setInterval(conditionalRefresh, 60 * 60 * 1000)
    return () => clearInterval(interval)
  }, [lastRefresh, token, auth])
  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Router>
        <Nav handleThemeChange={handleThemeChange} onLogout={handleLogout} />
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
          <AuthRoute exact path="/admin" auth={auth}>
            <AdminOnly user={user}>
              <AdminDashboard token={token} />
            </AdminOnly>
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

function AdminOnly({
  user,
  children,
}: {
  user: User | null
  children: ReactNode
}) {
  return <>{user?.access === Access.Admin ? children : <></>}</>
}
