import axios from "axios"
import httpStatusCodes from "http-status-codes"
import * as t from "io-ts"
import { API_ROOT } from "./config"
import { createEnumType } from "./io-validation"

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

const UserV = t.type({
  id: t.number,
  email: t.string,
  access: createEnumType<Access>(Access, "User"),
  password_hash: t.string,
})

export type User = t.TypeOf<typeof UserV>

export async function tokenFetcher(cred: EmailPassword): Promise<string> {
  const res = await axios.post(`${API_ROOT}/auth/session-token`, cred, {
    validateStatus: (s: number) =>
      [httpStatusCodes.OK, httpStatusCodes.UNAUTHORIZED].includes(s),
  })
  if (typeof res.data !== "string") {
    throw Error("unexpected response data: " + JSON.stringify(res.data))
  }
  if (res.status !== httpStatusCodes.OK) {
    if (res.data.startsWith("NoSuchUserEmail")) {
      throw Error(LoginFailure.EmailNotFound)
    }
    if (res.data.startsWith("WrongPassword")) {
      throw Error(LoginFailure.WrongPassword)
    }
    throw Error(res.data)
  }
  return res.data
}

export async function tokenValidator(tok: string): Promise<User> {
  const res = await axios.get(`${API_ROOT}/get/user/by/token/${tok}`, {
    validateStatus: (s) =>
      [httpStatusCodes.OK, httpStatusCodes.UNAUTHORIZED].includes(s),
  })
  if (res.status !== httpStatusCodes.OK) {
    throw Error(res.data)
  }
  if (!UserV.is(res.data)) {
    throw Error("unexpected response data: " + JSON.stringify(res.data))
  }
  return res.data as User
}
