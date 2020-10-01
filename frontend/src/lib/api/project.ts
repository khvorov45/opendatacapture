/** Project manipulation */

import * as t from "io-ts"
import { DateFromISOString } from "io-ts-types"
import axios from "axios"
import httpStatusCodes from "http-status-codes"
import { API_ROOT } from "../config"
import { decode } from "./io-validation"

const ProjectV = t.type({
  user: t.number,
  name: t.string,
  created: DateFromISOString,
})
export type Project = t.TypeOf<typeof ProjectV>

const ForeignKeyV = t.type({
  table: t.string,
  column: t.string,
})
export type ForeignKey = t.TypeOf<typeof ForeignKeyV>

const ColMetaV = t.type({
  name: t.string,
  postgres_type: t.string,
  not_null: t.boolean,
  unique: t.boolean,
  primary_key: t.boolean,
  foreign_key: t.union([ForeignKeyV, t.null]),
})
export type ColMeta = t.TypeOf<typeof ColMetaV>
export type ColSpec = ColMeta[]

const TableMetaV = t.type({
  name: t.string,
  cols: t.array(ColMetaV),
})
export type TableMeta = t.TypeOf<typeof TableMetaV>
export type TableSpec = TableMeta[]

export type TableData = Object[]

export async function createProject(tok: string, name: string): Promise<void> {
  const res = await axios.put(
    `${API_ROOT}/create/project/${name}`,
    {},
    {
      validateStatus: (s) =>
        [
          httpStatusCodes.NO_CONTENT,
          httpStatusCodes.CONFLICT,
          httpStatusCodes.UNAUTHORIZED,
        ].includes(s),
      headers: { Authorization: `Bearer ${tok}` },
    }
  )
  if (res.status !== httpStatusCodes.NO_CONTENT) {
    throw Error(res.data)
  }
}

export async function deleteProject(tok: string, name: string): Promise<void> {
  let res = await axios.delete(`${API_ROOT}/delete/project/${name}`, {
    validateStatus: (s) =>
      [
        httpStatusCodes.NO_CONTENT,
        httpStatusCodes.NOT_FOUND,
        httpStatusCodes.UNAUTHORIZED,
      ].includes(s),
    headers: { Authorization: `Bearer ${tok}` },
  })
  if (res.status !== httpStatusCodes.NO_CONTENT) {
    throw Error(res.data)
  }
}

export async function getUserProjects(tok: string): Promise<Project[]> {
  let res = await axios.get(`${API_ROOT}/get/projects`, {
    validateStatus: (s) =>
      [httpStatusCodes.OK, httpStatusCodes.UNAUTHORIZED].includes(s),
    headers: { Authorization: `Bearer ${tok}` },
  })
  if (res.status !== httpStatusCodes.OK) {
    throw Error(res.data)
  }
  return await decode(t.array(ProjectV), res.data)
}

export async function createTable(
  tok: string,
  projectName: string,
  tableMeta: TableMeta
): Promise<void> {
  let res = await axios.put(
    `${API_ROOT}/project/${projectName}/create/table`,
    tableMeta,
    {
      validateStatus: (s) =>
        [
          httpStatusCodes.NO_CONTENT,
          httpStatusCodes.NOT_FOUND,
          httpStatusCodes.UNAUTHORIZED,
          httpStatusCodes.CONFLICT,
        ].includes(s),
      headers: { Authorization: `Bearer ${tok}` },
    }
  )
  if (res.status !== httpStatusCodes.NO_CONTENT) {
    throw Error(res.data)
  }
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
        [
          httpStatusCodes.NO_CONTENT,
          httpStatusCodes.NOT_FOUND,
          httpStatusCodes.UNAUTHORIZED,
        ].includes(s),
      headers: { Authorization: `Bearer ${tok}` },
    }
  )
  if (res.status !== httpStatusCodes.NO_CONTENT) {
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
      validateStatus: (s) =>
        [
          httpStatusCodes.OK,
          httpStatusCodes.UNAUTHORIZED,
          httpStatusCodes.NOT_FOUND,
        ].includes(s),
      headers: { Authorization: `Bearer ${tok}` },
    }
  )
  if (res.status !== httpStatusCodes.OK) {
    throw Error(res.data)
  }
  return await decode(t.array(t.string), res.data)
}

export async function getAllMeta(
  tok: string,
  projectName: string
): Promise<TableSpec> {
  let res = await axios.get(`${API_ROOT}/project/${projectName}/get/meta`, {
    validateStatus: (s) =>
      [
        httpStatusCodes.OK,
        httpStatusCodes.UNAUTHORIZED,
        httpStatusCodes.NOT_FOUND,
      ].includes(s),
    headers: { Authorization: `Bearer ${tok}` },
  })
  if (res.status !== httpStatusCodes.OK) {
    throw Error(res.data)
  }
  return await decode(t.array(TableMetaV), res.data)
}

export async function getTableMeta(
  tok: string,
  projectName: string,
  tableName: string
): Promise<TableMeta> {
  let res = await axios.get(
    `${API_ROOT}/project/${projectName}/get/table/${tableName}/meta`,
    {
      validateStatus: (s) =>
        [
          httpStatusCodes.OK,
          httpStatusCodes.UNAUTHORIZED,
          httpStatusCodes.NOT_FOUND,
        ].includes(s),
      headers: { Authorization: `Bearer ${tok}` },
    }
  )
  if (res.status !== httpStatusCodes.OK) {
    throw Error(res.data)
  }
  return await decode(TableMetaV, res.data)
}

export async function insertData(
  tok: string,
  projectName: string,
  tableName: string,
  tableData: TableData
): Promise<void> {
  let res = await axios.put(
    `${API_ROOT}/project/${projectName}/insert/${tableName}`,
    tableData,
    {
      validateStatus: (s) =>
        [
          httpStatusCodes.NO_CONTENT,
          httpStatusCodes.UNAUTHORIZED,
          httpStatusCodes.NOT_FOUND,
          httpStatusCodes.CONFLICT,
        ].includes(s),
      headers: { Authorization: `Bearer ${tok}` },
    }
  )
  if (res.status !== httpStatusCodes.NO_CONTENT) {
    throw Error(res.data)
  }
}

export async function removeAllTableData(
  tok: string,
  projectName: string,
  tableName: string
): Promise<void> {
  let res = await axios.delete(
    `${API_ROOT}/project/${projectName}/remove/${tableName}/all`,
    {
      validateStatus: (s) =>
        [
          httpStatusCodes.NO_CONTENT,
          httpStatusCodes.UNAUTHORIZED,
          httpStatusCodes.NOT_FOUND,
        ].includes(s),
      headers: { Authorization: `Bearer ${tok}` },
    }
  )
  if (res.status !== httpStatusCodes.NO_CONTENT) {
    throw Error(res.data)
  }
}

export async function getTableData(
  tok: string,
  projectName: string,
  tableName: string
): Promise<TableData> {
  let res = await axios.get(
    `${API_ROOT}/project/${projectName}/get/table/${tableName}/data`,
    {
      validateStatus: (s) =>
        [
          httpStatusCodes.OK,
          httpStatusCodes.UNAUTHORIZED,
          httpStatusCodes.NOT_FOUND,
        ].includes(s),
      headers: { Authorization: `Bearer ${tok}` },
    }
  )
  if (res.status !== httpStatusCodes.OK) {
    throw Error(res.data)
  }
  return await decode(t.array(t.UnknownRecord), res.data)
}
