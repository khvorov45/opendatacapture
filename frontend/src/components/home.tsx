import React, { useState, useEffect } from "react"
import { Token, Access } from "../lib/auth"
import { Redirect } from "react-router-dom"

export default function Home({
  token,
  tokenValidator,
}: {
  token: Token | null
  tokenValidator: (t: Token) => Promise<Access>
}) {
  const [access, setAccess] = useState<Access | null>(null)
  useEffect(() => {
    if (!token) return
    tokenValidator(token).then((a) => setAccess(a))
  }, [token, tokenValidator])
  if (!token || access === Access.Unauthorized) {
    return <Redirect to="/login" />
  }
  if (!access) {
    return <></>
  }
  return (
    <p>
      This is the homepage for user id {token.user} with access {access}
    </p>
  )
}
