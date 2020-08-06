import React from "react"
import { Token } from "../lib/auth"
import { Redirect } from "react-router-dom"

export default function Home({ token }: { token: Token | null }) {
  if (!token) {
    return <Redirect to="/login" />
  }
  return <p>This is the homepage</p>
}
