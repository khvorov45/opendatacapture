import React, { ReactNode, useState } from "react"
import { createMuiTheme, ThemeProvider, Theme } from "@material-ui/core/styles"
import Nav from "./components/nav"
import Login from "./components/login"
import Project from "./components/project"
import { tokenFetcher, tokenValidator } from "./lib/auth"
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
  const { auth } = useToken(token, tokenValidator)
  const [currentProject, setCurrentProject] = useState<string | undefined>()
  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Nav
        handleThemeChange={handleThemeChange}
        currentProject={currentProject}
      />
      <Router>
        <Switch>
          <Route exact path="/login">
            {auth === AuthStatus.Ok ? (
              <Redirect to="/" />
            ) : (
              <Login updateToken={updateToken} tokenFetcher={tokenFetcher} />
            )}
          </Route>
          <AuthRoute exact path="/" auth={auth}>
            <Home token={token} />
          </AuthRoute>
          <AuthRoute path="/project/:name" auth={auth}>
            <Project
              onVisit={(projectName: string) => setCurrentProject(projectName)}
            />
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
