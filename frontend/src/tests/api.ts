/* istanbul ignore file */
/** Mocked API calls for tests */

import httpStatusCodes from "http-status-codes"
import { defaultAdmin } from "./util"

export async function getOk(url: string) {
  if (url.endsWith("/get/users")) {
    return await getUsers()
  }
}

export async function getUsers() {
  return { status: httpStatusCodes.OK, data: [defaultAdmin] }
}
