/* istanbul ignore file */
/** Mocked API calls for tests */

import httpStatusCodes from "http-status-codes"
import { defaultAdmin } from "./util"

const defaultGet = {
  getUsers: async () => ({ status: httpStatusCodes.OK, data: [defaultAdmin] }),
}

/** Whatever is in `fns` is supposed to overwrite `defaultGet` */
export function constructGet(fns?: Record<string, any>) {
  const currentGet = Object.assign(defaultGet, fns)
  const mockedGet = async (url: string) => {
    if (url.endsWith("/get/users")) {
      return await currentGet.getUsers()
    }
    throw Error("unimplemented path in mocked get")
  }
  return mockedGet
}
