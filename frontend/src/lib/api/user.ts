import axios from "axios"
import httpStatusCodes from "http-status-codes"
import * as t from "io-ts"
import { API_ROOT } from "../config"
import { User, UserV } from "./auth"
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
