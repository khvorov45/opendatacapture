import axios from "axios"
import httpStatusCodes from "http-status-codes"
import * as t from "io-ts"
import { DateFromISOString } from "io-ts-types"
import { API_ROOT } from "../config"
import { decode, fromEnum } from "./io-validation"

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

export const UserV = t.type({
  id: t.number,
  email: t.string,
  access: fromEnum<Access>("Access", Access),
})
export type User = t.TypeOf<typeof UserV>

export const TokenV = t.type({
  user: t.number,
  token: t.string,
  created: DateFromISOString,
})
export type Token = t.TypeOf<typeof TokenV>

export async function fetchToken(cred: EmailPassword): Promise<Token> {
  const res = await axios.post(`${API_ROOT}/auth/session-token`, cred, {
    validateStatus: (s: number) =>
      [httpStatusCodes.OK, httpStatusCodes.UNAUTHORIZED].includes(s),
  })
  if (res.status !== httpStatusCodes.OK) {
    if (res.data.startsWith("NoSuchUserEmail")) {
      throw Error(LoginFailure.EmailNotFound)
    }
    if (res.data.startsWith("WrongPassword")) {
      throw Error(LoginFailure.WrongPassword)
    }
    throw Error(res.data)
  }
  return await decode(TokenV, res.data)
}

export async function validateToken(tok: string): Promise<User> {
  const res = await axios.get(`${API_ROOT}/get/user/by/token/${tok}`, {
    validateStatus: (s) =>
      [httpStatusCodes.OK, httpStatusCodes.UNAUTHORIZED].includes(s),
  })
  if (res.status !== httpStatusCodes.OK) {
    throw Error(res.data)
  }
  return await decode(UserV, res.data)
}

export async function refreshToken(tok: string): Promise<Token> {
  const res = await axios.post(
    `${API_ROOT}/auth/refresh-token/${tok}`,
    undefined,
    {
      validateStatus: (s: number) =>
        [httpStatusCodes.OK, httpStatusCodes.UNAUTHORIZED].includes(s),
    }
  )
  if (res.status !== httpStatusCodes.OK) {
    throw Error(res.data)
  }
  return await decode(TokenV, res.data)
}

export async function removeToken(tok: string): Promise<void> {
  await axios.delete(`${API_ROOT}/auth/remove-token/${tok}`, {
    validateStatus: (s: number) => [httpStatusCodes.NO_CONTENT].includes(s),
  })
}
