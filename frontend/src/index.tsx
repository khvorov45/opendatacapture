import React from "react"
import ReactDOM from "react-dom"
import "./index.css"
import App from "./App"
import * as serviceWorker from "./serviceWorker"
import { themeInit } from "./lib/theme"

/* istanbul ignore file */

// Work out the theme before rendering
const initPalette = themeInit()

// Work out the initial token
const initToken = localStorage.getItem("token")
const lastRefresh = new Date(localStorage.getItem("last-refresh") ?? 0)

ReactDOM.render(
  <React.StrictMode>
    <App
      initPalette={initPalette}
      initToken={initToken}
      initLastRefresh={lastRefresh}
    />
  </React.StrictMode>,
  document.getElementById("root")
)

// If you want your app to work offline and load faster, you can change
// unregister() to register() below. Note this comes with some pitfalls.
// Learn more about service workers: https://bit.ly/CRA-PWA
serviceWorker.unregister()
