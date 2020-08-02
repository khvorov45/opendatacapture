import React, { useState } from "react"
import { createMuiTheme, ThemeProvider } from "@material-ui/core/styles"
import "./App.css"
import Switch from "@material-ui/core/Switch"

function App() {
  const [darkState, setDarkState] = useState(
    localStorage.getItem("theme") === "dark"
  )
  const palletType = darkState ? "dark" : "light"
  const darkTheme = createMuiTheme({
    palette: {
      type: palletType,
    },
  })
  const handleThemeChange = () => {
    let newDarkState = !darkState
    let newDarkStateName = newDarkState ? "dark" : "light"
    setDarkState(newDarkState)
    document.documentElement.setAttribute("theme", newDarkStateName)
    localStorage.setItem("theme", newDarkStateName)
  }
  return (
    <div className="App">
      <ThemeProvider theme={darkTheme}>
        <Switch checked={darkState} onChange={handleThemeChange} />
      </ThemeProvider>
    </div>
  )
}

export default App
