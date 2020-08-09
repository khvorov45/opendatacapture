import axios from "axios"

export interface EmailPassword {
  email: string
  password: string
}

export interface IdToken {
  id: number
  token: string
}

export interface Token {
  user: number
  token: string
  created: Date
}

export function tokenFromObject(o: any): Token {
  if (!o || !o.user || !o.token || !o.created) {
    throw Error("cannot create token from object: " + JSON.stringify(o))
  }
  const created = new Date(o.created)
  if (isNaN(created.getTime())) {
    throw Error("invalid date: " + JSON.stringify(o.created))
  }
  return {
    user: o.user,
    token: o.token,
    created: created,
  }
}

export function tokenFromString(s: string): Token {
  return tokenFromObject(JSON.parse(s))
}

export function tokenInit(): Token | null {
  const initTokenString = localStorage.getItem("token")
  let initToken: Token | null = null
  if (initTokenString) {
    try {
      initToken = tokenFromString(initTokenString)
    } catch (e) {}
  }
  return initToken
}

export enum Access {
  Unauthorized,
  User,
  Admin,
}

export async function tokenFetcher(cred: EmailPassword): Promise<Token> {
  const res = await axios.post(
    "http://localhost:4321/auth/email-password",
    cred
  )
  // Should be an object if successful
  if (typeof res.data === "string") {
    throw Error(res.data)
  }
  return tokenFromObject(res.data)
}

export async function tokenValidator(tok: Token): Promise<Access> {
  // @UNIMPLEMENTED
  return Access.Admin
}
