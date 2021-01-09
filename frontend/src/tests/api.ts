/* istanbul ignore file */
/** Mocked API calls for tests */

import httpStatusCodes from "http-status-codes"
import { TokenV, UserV } from "../lib/api/auth"
import { ProjectV } from "../lib/api/project"
import { adminToken, allTables, defaultAdmin, project1 } from "./data"

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

type RequestFns = Record<
  string,
  (...args: any[]) => Promise<{ status: number; data?: any }>
>

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
  validateToken: async () => ({
    status: httpStatusCodes.OK,
    data: UserV.encode(defaultAdmin),
  }),
  getUserProjects: async () => ({
    status: httpStatusCodes.OK,
    data: [ProjectV.encode(project1)],
  }),
  getAllMeta: async () => ({
    status: httpStatusCodes.OK,
    data: allTables.map((t) => t.meta),
  }),
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
    if (url.includes("get/projects")) {
      return await currentGet.getUserProjects()
    }
    if (url.includes("/get/meta")) {
      return await currentGet.getAllMeta()
    }
    throw Error("unimplemented path in mocked get")
  }
  return mockedGet
}

export const defaultDelete: RequestFns = {
  removeUser: async () => ({ status: httpStatusCodes.NO_CONTENT }),
  removeToken: async () => ({ status: httpStatusCodes.NO_CONTENT }),
  deleteProject: async () => ({ status: httpStatusCodes.NO_CONTENT }),
  removeTable: async () => ({ status: httpStatusCodes.NO_CONTENT }),
  removeAllTableData: async () => ({ status: httpStatusCodes.NO_CONTENT }),
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
    if (url.includes("/delete/project")) {
      return await currentDelete.deleteProject()
    }
    if (url.includes("/remove/table")) {
      return await currentDelete.removeTable()
    }
    if (url.match("/project/.*/remove/.*/all")) {
      return await currentDelete.removeAllTableData()
    }
    throw Error("unimplemented path in mocked delete")
  }
  return mockedDelete
}

export const defaultPut: RequestFns = {
  createUser: async () => ({ status: httpStatusCodes.NO_CONTENT }),
  createProject: async () => ({ status: httpStatusCodes.NO_CONTENT }),
  createTable: async () => ({ status: httpStatusCodes.NO_CONTENT }),
  insertData: async () => ({ status: httpStatusCodes.NO_CONTENT }),
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
    if (url.includes("create/table")) {
      return await currentPut.createTable()
    }
    if (url.match("/project/.*/insert/.*")) {
      return await currentPut.insertData()
    }
    throw Error("unimplemented path in mocked put")
  }
  return mockedPut
}

export const defaultPost: RequestFns = {
  fetchToken: async () => ({
    status: httpStatusCodes.OK,
    data: TokenV.encode(adminToken),
  }),
  refreshToken: async () => ({
    status: httpStatusCodes.OK,
    data: TokenV.encode(adminToken),
  }),
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
