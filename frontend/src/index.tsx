import React from "react"
import ReactDOM from "react-dom"
import "./index.css"
import App from "./App"
import * as serviceWorker from "./serviceWorker"
import { themeInit } from "./lib/theme"

/* istanbul ignore file */

// Work out the initial token
const initToken = localStorage.getItem("token")

ReactDOM.render(
  <React.StrictMode>
    <App initToken={initToken} />
  </React.StrictMode>,
  document.getElementById("root")
)

// If you want your app to work offline and load faster, you can change
// unregister() to register() below. Note this comes with some pitfalls.
// Learn more about service workers: https://bit.ly/CRA-PWA
serviceWorker.unregister()
