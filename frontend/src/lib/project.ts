/** Project manipulation */

import axios from "axios"
import httpStatusCodes from "http-status-codes"

export interface Project {
  user: number
  name: string
  created: Date
}

export async function createProject(tok: string, name: string): Promise<void> {
  await axios.put(
    `http://localhost:4321/create/project/${name}`,
    {},
    {
      validateStatus: (s) => s === httpStatusCodes.OK,
      headers: { Authorization: `Bearer ${tok}` },
    }
  )
}

export async function deleteProject(tok: string, name: string): Promise<void> {
  await axios.delete(`http://localhost:4321/delete/project/${name}`, {
    validateStatus: (s) => s === httpStatusCodes.OK,
    headers: { Authorization: `Bearer ${tok}` },
  })
}

export async function getUserProjects(tok: string): Promise<Project[]> {
  let res = await axios.get(`http://localhost:4321/get/projects`, {
    validateStatus: (s) => s === httpStatusCodes.OK,
    headers: { Authorization: `Bearer ${tok}` },
  })
  return res.data
}
