import React, { useState, useEffect } from "react"
import { User } from "../lib/auth"
import { Redirect } from "react-router-dom"

enum AuthStatus {
  InProgress,
  Ok,
  Err,
}

function useToken(
  token: string | null,
  tokenValidator: (s: string) => Promise<User>
): { user: User | null; auth: AuthStatus } {
  const [user, setUser] = useState<User | null>(null)
  const [auth, setAuth] = useState<AuthStatus>(AuthStatus.InProgress)
  useEffect(() => {
    if (!token) {
      setAuth(AuthStatus.Err)
      return
    }
    tokenValidator(token)
      .then((u) => {
        setUser(u)
        setAuth(AuthStatus.Ok)
      })
      .catch((e) => setAuth(AuthStatus.Err))
  }, [token, tokenValidator])
  return { user: user, auth: auth }
}

export default function Home({
  token,
  tokenValidator,
}: {
  token: string | null
  tokenValidator: (t: string) => Promise<User>
}) {
  const { user, auth } = useToken(token, tokenValidator)
  if (auth === AuthStatus.Err) {
    return <Redirect to="/login" />
  }
  if (!user || auth === AuthStatus.InProgress) {
    return <></>
  }
  return (
    <div data-testid="homepage">
      This is the homepage for user {user.email} with access {user.access}
    </div>
  )
}
