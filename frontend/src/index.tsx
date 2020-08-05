import React from "react"
import ReactDOM from "react-dom"
import "./index.css"
import App from "./App"
import * as serviceWorker from "./serviceWorker"
import { themeInit } from "./lib/theme"

// Work out the theme before rendering
const initPalette = themeInit()

ReactDOM.render(
  <React.StrictMode>
    <App initPalette={initPalette} />
  </React.StrictMode>,
  document.getElementById("root")
)

// If you want your app to work offline and load faster, you can change
// unregister() to register() below. Note this comes with some pitfalls.
// Learn more about service workers: https://bit.ly/CRA-PWA
serviceWorker.unregister()
