/* istanbul ignore file */
/** Mocked API calls for tests */

import httpStatusCodes from "http-status-codes"
import { adminToken, allTables, defaultAdmin } from "./data"

function findTableEntry(tableName: string) {
  const tableEntry = allTables.filter((t) => t.meta.name === tableName)
  if (tableEntry.length !== 1) {
    throw Error(
      `want to find entry for '${tableName}'
      but there is no one such test table`
    )
  }
  return tableEntry[0]
}

type RequestFns = Record<string, (...[]: any[]) => Promise<any>>

export const defaultGet: RequestFns = {
  getUsers: async () => ({ status: httpStatusCodes.OK, data: [defaultAdmin] }),
  getAllTableNames: async () => ({
    status: httpStatusCodes.OK,
    data: allTables.map((t) => t.meta.name),
  }),
  getTableData: async (tableName: string) => ({
    status: httpStatusCodes.OK,
    data: findTableEntry(tableName).data,
  }),
  getTableMeta: async (tableName: string) => ({
    status: httpStatusCodes.OK,
    data: findTableEntry(tableName).meta,
  }),
  validateToken: async () => defaultAdmin,
}

/** Whatever is in `fns` is supposed to overwrite `defaultGet` */
export function constructGet(fns?: RequestFns) {
  const currentGet = Object.assign({ ...defaultGet }, fns)
  const mockedGet = async (url: string) => {
    if (url.endsWith("/get/users")) {
      return await currentGet.getUsers()
    }
    if (url.endsWith("/get/tablenames")) {
      return await currentGet.getAllTableNames()
    }
    const tableDataMatch = url.match("/get/table/(.*)/data")
    if (tableDataMatch) {
      return await currentGet.getTableData(tableDataMatch[1])
    }
    const tableMetaMatch = url.match("/get/table/(.*)/meta")
    if (tableMetaMatch) {
      return await currentGet.getTableMeta(tableMetaMatch[1])
    }
    if (url.includes("/get/user/by/token/")) {
      return await currentGet.validateToken()
    }
    throw Error("unimplemented path in mocked get")
  }
  return mockedGet
}

export const defaultDelete: RequestFns = {
  removeUser: async () => ({ status: httpStatusCodes.NO_CONTENT }),
  removeToken: async () => ({ status: httpStatusCodes.NO_CONTENT }),
}

export function constructDelete(fns?: RequestFns) {
  const currentDelete = Object.assign({ ...defaultDelete }, fns)
  const mockedDelete = async (url: string) => {
    if (url.includes("/remove/user/")) {
      return await currentDelete.removeUser()
    }
    if (url.includes("/auth/remove-token/")) {
      return await currentDelete.removeToken()
    }
    throw Error("unimplemented path in mocked delete")
  }
  return mockedDelete
}

export const defaultPut: RequestFns = {
  createUser: async () => ({ status: httpStatusCodes.NO_CONTENT }),
  createProject: async () => ({ status: httpStatusCodes.NO_CONTENT }),
}

export function constructPut(fns?: RequestFns) {
  const currentPut = Object.assign({ ...defaultPut }, fns)
  const mockedPut = async (url: string) => {
    if (url.endsWith("/create/user")) {
      return await currentPut.createUser()
    }
    if (url.includes("create/project")) {
      return await currentPut.createProject()
    }
    throw Error("unimplemented path in mocked put")
  }
  return mockedPut
}

export const defaultPost: RequestFns = {
  fetchToken: async () => ({ status: httpStatusCodes.OK, data: adminToken }),
  refreshToken: async () => adminToken,
}

export function constructPost(fns?: RequestFns) {
  const currentPost = Object.assign({ ...defaultPost }, fns)
  const mockedPost = async (url: string) => {
    if (url.endsWith("/auth/session-token")) {
      return await currentPost.fetchToken()
    }
    if (url.includes("/auth/refresh-token/")) {
      return await currentPost.refreshToken()
    }
    throw Error("unimplemented path in mocked post")
  }
  return mockedPost
}
