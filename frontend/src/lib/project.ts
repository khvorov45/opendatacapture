/** Project manipulation */

import axios from "axios"
import httpStatusCodes from "http-status-codes"

import { API_ROOT } from "./config"

export interface Project {
  user: number
  name: string
  created: Date
}

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
  await axios.delete(`${API_ROOT}/delete/project/${name}`, {
    validateStatus: (s) => s === httpStatusCodes.OK,
    headers: { Authorization: `Bearer ${tok}` },
  })
}

export async function getUserProjects(tok: string): Promise<Project[]> {
  let res = await axios.get(`${API_ROOT}/get/projects`, {
    validateStatus: (s) => s === httpStatusCodes.OK,
    headers: { Authorization: `Bearer ${tok}` },
  })
  return res.data
}
