import React, { useState } from "react"
import { createMuiTheme, ThemeProvider, Theme } from "@material-ui/core/styles"
import Nav from "./components/nav"

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
  return (
    <div>
      <ThemeProvider theme={theme}>
        <Nav darkState={darkState} handleThemeChange={handleThemeChange} />
      </ThemeProvider>
    </div>
  )
}
