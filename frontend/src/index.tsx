import React from "react"
import ReactDOM from "react-dom"
import "./index.css"
import App from "./App"
import * as serviceWorker from "./serviceWorker"
import { themeInit } from "./lib/theme"
import { Token } from "./lib/auth"

// Work out the theme before rendering
const initPalette = themeInit()
const initTokenString = localStorage.getItem("token")

ReactDOM.render(
  <React.StrictMode>
    <App
      initPalette={initPalette}
      initToken={
        initTokenString ? (JSON.parse(initTokenString) as Token) : null
      }
    />
  </React.StrictMode>,
  document.getElementById("root")
)

// If you want your app to work offline and load faster, you can change
// unregister() to register() below. Note this comes with some pitfalls.
// Learn more about service workers: https://bit.ly/CRA-PWA
serviceWorker.unregister()
