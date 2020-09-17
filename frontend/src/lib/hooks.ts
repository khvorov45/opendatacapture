import { User } from "../lib/auth"
import { useState, useEffect } from "react"
import { useLocation } from "react-router-dom"

export enum AuthStatus {
  InProgress,
  Ok,
  Err,
}

export function useToken(
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
