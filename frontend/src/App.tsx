import React, { useState } from "react"
import { createMuiTheme, ThemeProvider, Theme } from "@material-ui/core/styles"
import Nav from "./components/nav"
import Login from "./components/login"
import { IdToken } from "./lib/auth"
import CssBaseline from "@material-ui/core/CssBaseline"

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
  function updateCred(cred: IdToken) {
    console.log(cred)
  }
  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Nav handleThemeChange={handleThemeChange} />
      <Login updateCred={updateCred} />
    </ThemeProvider>
  )
}
