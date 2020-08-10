import axios from "axios"
import httpStatusCodes from "http-status-codes"

export interface EmailPassword {
  email: string
  password: string
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

export async function tokenFetcher(cred: EmailPassword): Promise<string> {
  const res = await axios.post(
    "http://localhost:4321/auth/session-token",
    cred,
    {
      validateStatus: (s: number) =>
        [
          httpStatusCodes.OK,
          httpStatusCodes.UNAUTHORIZED,
          httpStatusCodes.INTERNAL_SERVER_ERROR,
        ].includes(s),
    }
  )
  if (typeof res.data !== "string") {
    throw Error(`unexpected response data: ${JSON.stringify(res.data)}`)
  }
  if (res.status !== httpStatusCodes.OK) {
    if (res.data.startsWith("NoSuchUser")) {
      throw Error("EmailNotFound")
    }
    if (res.data.startsWith("WrongPassword")) {
      throw Error("WrongPassword")
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
  const res = await axios.get(`http://localhost:4321/get/user/by/token/${tok}`)
  return res.data as User
}
