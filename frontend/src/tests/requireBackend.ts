/** All of these tests assume that the backend is running as if it was
 * started with the --clean option
 */

/* istanbul ignore file */

import { Access, LoginFailure, tokenFetcher, tokenValidator } from "../lib/auth"
import { createProject, getUserProjects, deleteProject } from "../lib/project"

test("wrong token", async () => {
  expect.assertions(1)
  try {
    let res = await tokenValidator("123")
    console.log(`wrong token response: ${res}`)
  } catch (e) {
    expect(e.message).toBe("Request failed with status code 401")
  }
})

test("wrong password", async () => {
  expect.assertions(1)
  try {
    let res = await tokenFetcher({
      email: "admin@example.com",
      password: "123",
    })
    console.log(`wrong password response: ${res}`)
  } catch (e) {
    expect(e.message).toBe(LoginFailure.WrongPassword)
  }
})

test("wrong email", async () => {
  expect.assertions(1)
  try {
    let res = await tokenFetcher({
      email: "user@example.com",
      password: "admin",
    })
    console.log(`wrong email response: ${res}`)
  } catch (e) {
    expect(e.message).toBe(LoginFailure.EmailNotFound)
  }
})

test("correct credentials", async () => {
  let token = await tokenFetcher({
    email: "admin@example.com",
    password: "admin",
  })
  let admin = await tokenValidator(token)
  expect(admin.access).toBe(Access.Admin)
  expect(admin.email).toBe("admin@example.com")
  expect(admin.id).toBe(1)
})

test("project manipulation", async () => {
  let token = await tokenFetcher({
    email: "admin@example.com",
    password: "admin",
  })
  await createProject(token, "test")
  let projects = await getUserProjects(token)
  let projectIds = projects.map((p) => `${p.user}${p.name}`)
  expect(projectIds).toContain("1test")
  await deleteProject(token, "test")
  projects = await getUserProjects(token)
  projectIds = projects.map((p) => `${p.user}${p.name}`)
  expect(projectIds).not.toContain("1test")
})

test("create the same project twice", async () => {
  expect.assertions(1)
  let token = await tokenFetcher({
    email: "admin@example.com",
    password: "admin",
  })
  await createProject(token, "test")
  try {
    await createProject(token, "test")
  } catch (e) {
    expect(e.message).toBe('ProjectAlreadyExists(1, "test")')
  }
  await deleteProject(token, "test")
})
