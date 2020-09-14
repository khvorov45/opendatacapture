/** Project manipulation */

import axios from "axios"
import httpStatusCodes from "http-status-codes"

import { API_ROOT } from "./config"

export interface Project {
  user: number
  name: string
  created: Date
}

export type TableSpec = TableMeta[]

export interface TableMeta {
  name: string
  cols: ColSpec
}

export type ColSpec = ColMeta[]

export interface ColMeta {
  name: string
  postgres_type: string
  not_null: boolean
  unique: boolean
  primary_key: boolean
  foreign_key: ForeignKey | null
}

export interface ForeignKey {
  table: string
  column: string
}

export type TableData = Object[]

export async function createProject(tok: string, name: string): Promise<void> {
  const res = await axios.put(
    `${API_ROOT}/create/project/${name}`,
    {},
    {
      validateStatus: (s) =>
        [httpStatusCodes.OK, httpStatusCodes.CONFLICT].includes(s),
      headers: { Authorization: `Bearer ${tok}` },
    }
  )
  if (res.status !== httpStatusCodes.OK) {
    throw Error(res.data)
  }
}

export async function deleteProject(tok: string, name: string): Promise<void> {
  let res = await axios.delete(`${API_ROOT}/delete/project/${name}`, {
    validateStatus: (s) =>
      [httpStatusCodes.NO_CONTENT, httpStatusCodes.NOT_FOUND].includes(s),
    headers: { Authorization: `Bearer ${tok}` },
  })
  if (res.status === httpStatusCodes.NOT_FOUND) {
    throw Error(res.data)
  }
}

export async function getUserProjects(tok: string): Promise<Project[]> {
  let res = await axios.get(`${API_ROOT}/get/projects`, {
    validateStatus: (s) => s === httpStatusCodes.OK,
    headers: { Authorization: `Bearer ${tok}` },
  })
  return res.data
}

export async function createTable(
  tok: string,
  projectName: string,
  tableMeta: TableMeta
): Promise<void> {
  await axios.put(
    `${API_ROOT}/project/${projectName}/create/table`,
    tableMeta,
    {
      validateStatus: (s) => s === httpStatusCodes.OK,
      headers: { Authorization: `Bearer ${tok}` },
    }
  )
}

export async function removeTable(
  tok: string,
  projectName: string,
  tableName: string
): Promise<void> {
  let res = await axios.delete(
    `${API_ROOT}/project/${projectName}/remove/table/${tableName}`,
    {
      validateStatus: (s) =>
        [httpStatusCodes.NO_CONTENT, httpStatusCodes.NOT_FOUND].includes(s),
      headers: { Authorization: `Bearer ${tok}` },
    }
  )
  if (res.status === httpStatusCodes.NOT_FOUND) {
    throw Error(res.data)
  }
}

export async function getAllTableNames(
  tok: string,
  projectName: string
): Promise<string[]> {
  "/project/test/get/tablenames"
  let res = await axios.get(
    `${API_ROOT}/project/${projectName}/get/tablenames`,
    {
      validateStatus: (s) => s === httpStatusCodes.OK,
      headers: { Authorization: `Bearer ${tok}` },
    }
  )
  return res.data
}

export async function getAllMeta(
  tok: string,
  projectName: string
): Promise<TableSpec> {
  let res = await axios.get(`${API_ROOT}/project/${projectName}/get/meta`, {
    validateStatus: (s) => s === httpStatusCodes.OK,
    headers: { Authorization: `Bearer ${tok}` },
  })
  return res.data
}

export async function getTableMeta(
  tok: string,
  projectName: string,
  tableName: string
): Promise<TableMeta> {
  let res = await axios.get(
    `${API_ROOT}/project/${projectName}/get/table/${tableName}/meta`,
    {
      validateStatus: (s) => s === httpStatusCodes.OK,
      headers: { Authorization: `Bearer ${tok}` },
    }
  )
  return res.data
}

export async function insertData(
  tok: string,
  projectName: string,
  tableName: string,
  tableData: TableData
): Promise<void> {
  await axios.put(
    `${API_ROOT}/project/${projectName}/insert/${tableName}`,
    tableData,
    {
      validateStatus: (s) => s === httpStatusCodes.OK,
      headers: { Authorization: `Bearer ${tok}` },
    }
  )
}

export async function removeAllTableData(
  tok: string,
  projectName: string,
  tableName: string
): Promise<void> {
  await axios.delete(
    `${API_ROOT}/project/${projectName}/remove/${tableName}/all`,
    {
      validateStatus: (s) => s === httpStatusCodes.OK,
      headers: { Authorization: `Bearer ${tok}` },
    }
  )
}

export async function getTableData(
  tok: string,
  projectName: string,
  tableName: string
): Promise<TableData> {
  let res = await axios.get(
    `${API_ROOT}/project/${projectName}/get/table/${tableName}/data`,
    {
      validateStatus: (s) => s === httpStatusCodes.OK,
      headers: { Authorization: `Bearer ${tok}` },
    }
  )
  return res.data
}
