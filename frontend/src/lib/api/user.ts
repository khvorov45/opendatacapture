import axios from "axios"
import httpStatusCodes from "http-status-codes"
import * as t from "io-ts"
import { API_ROOT } from "../config"
import { EmailPassword, User, UserV } from "./auth"
import { decode } from "./io-validation"

export async function getUsers(tok: string): Promise<User[]> {
  const res = await axios.get(`${API_ROOT}/get/users`, {
    validateStatus: (s) =>
      [httpStatusCodes.OK, httpStatusCodes.UNAUTHORIZED].includes(s),
    headers: { Authorization: `Bearer ${tok}` },
  })
  if (res.status !== httpStatusCodes.OK) {
    throw Error(res.data)
  }
  return await decode(t.array(UserV), res.data)
}

export async function createUser(newUser: EmailPassword): Promise<void> {
  const res = await axios.put(`${API_ROOT}/create/user`, newUser, {
    validateStatus: (s) => [httpStatusCodes.NO_CONTENT].includes(s),
  })
  if (res.status !== httpStatusCodes.NO_CONTENT) {
    throw Error(res.data)
  }
}

export async function removeUser(token: string, email: string): Promise<void> {
  const res = await axios.delete(`${API_ROOT}/remove/user/${email}`, {
    validateStatus: (s) => [httpStatusCodes.NO_CONTENT].includes(s),
    headers: { Authorization: `Bearer ${token}` },
  })
  if (res.status !== httpStatusCodes.NO_CONTENT) {
    throw Error(res.data)
  }
}
