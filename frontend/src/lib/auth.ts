import axios from "axios"
import httpStatusCodes from "http-status-codes"
import { API_ROOT } from "./config"

export interface EmailPassword {
  email: string
  password: string
}

export enum LoginFailure {
  EmailNotFound = "EmailNotFound",
  WrongPassword = "WrongPassword",
}

export enum Access {
  User = "User",
  Admin = "Admin",
}

export interface User {
  id: number
  email: string
  access: Access
  password_hash: string
}

function validateUser(u: any): boolean {
  return (
    u &&
    Object.keys(u).length === 4 &&
    typeof u.id === "number" &&
    typeof u.email === "string" &&
    [Access.User, Access.Admin].includes(u.access) &&
    typeof u.password_hash === "string"
  )
}

export async function tokenFetcher(cred: EmailPassword): Promise<string> {
  const res = await axios.post(`${API_ROOT}/auth/session-token`, cred, {
    validateStatus: (s: number) =>
      [
        httpStatusCodes.OK,
        httpStatusCodes.UNAUTHORIZED,
        httpStatusCodes.INTERNAL_SERVER_ERROR,
      ].includes(s),
  })
  if (typeof res.data !== "string") {
    throw Error(`unexpected response data: ${JSON.stringify(res.data)}`)
  }
  if (res.status !== httpStatusCodes.OK) {
    if (res.data.startsWith("NoSuchUserEmail")) {
      throw Error(LoginFailure.EmailNotFound)
    }
    if (res.data.startsWith("WrongPassword")) {
      throw Error(LoginFailure.WrongPassword)
    }
    throw Error(
      `login failed with status ${res.status} and data ${JSON.stringify(
        res.data
      )}`
    )
  }
  return res.data
}

export async function tokenValidator(tok: string): Promise<User> {
  const res = await axios.get(`${API_ROOT}/get/user/by/token/${tok}`, {
    validateStatus: (s) => s === httpStatusCodes.OK,
  })
  if (!validateUser(res.data)) {
    throw Error("unexpected response data: " + JSON.stringify(res.data))
  }
  return res.data as User
}
