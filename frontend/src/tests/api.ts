/* istanbul ignore file */
/** Mocked API calls for tests */

import httpStatusCodes from "http-status-codes"
import { defaultAdmin } from "./data"

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

const defaultDelete = {
  removeUser: async () => ({ status: httpStatusCodes.NO_CONTENT }),
}

export function constructDelete(fns?: Record<string, any>) {
  const currentDelete = Object.assign(defaultDelete, fns)
  const mockedDelete = async (url: string) => {
    if (url.includes("/remove/user/")) {
      return await currentDelete.removeUser()
    }
    throw Error("unimplemented path in mocked delete")
  }
  return mockedDelete
}

const defaultCreate = {
  createUser: async () => ({ status: httpStatusCodes.NO_CONTENT }),
}

export function constructPut(fns?: Record<string, any>) {
  const currentPut = Object.assign(defaultCreate, fns)
  const mockedPut = async (url: string) => {
    if (url.endsWith("/create/user")) {
      return await currentPut.createUser()
    }
    throw Error("unimplemented path in mocked put")
  }
  return mockedPut
}
