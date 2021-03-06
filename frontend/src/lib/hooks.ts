import { User } from "../lib/api/auth"
import { useState, useEffect } from "react"

export enum AuthStatus {
  InProgress,
  Ok,
  Err,
}

export function useToken(
  token: string | null,
  validateToken: (s: string) => Promise<User>
): { user: User | null; auth: AuthStatus } {
  const [user, setUser] = useState<User | null>(null)
  const [auth, setAuth] = useState<AuthStatus>(AuthStatus.InProgress)
  useEffect(() => {
    if (!token) {
      setAuth(AuthStatus.Err)
      return
    }
    validateToken(token)
      .then((u) => {
        setUser(u)
        setAuth(AuthStatus.Ok)
      })
      .catch((e) => setAuth(AuthStatus.Err))
  }, [token, validateToken])
  return { user: user, auth: auth }
}
