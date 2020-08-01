import React, { useState } from "react"
import { createMuiTheme, ThemeProvider } from "@material-ui/core/styles"
import "./App.css"
import Switch from "@material-ui/core/Switch"

function App() {
  const [darkState, setDarkState] = useState(true)
  const palletType = darkState ? "dark" : "light"
  const darkTheme = createMuiTheme({
    palette: {
      type: palletType,
    },
  })
  const handleThemeChange = () => {
    setDarkState(!darkState)
    document.documentElement.setAttribute("theme", darkState ? "dark" : "light")
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
