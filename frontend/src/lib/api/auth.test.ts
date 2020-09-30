// Test whatever can't be tested in fullstack
/* istanbul ignore file */
import httpStatusCodes from "http-status-codes"
import { tokenFetcher, tokenValidator } from "./auth"

import axios from "axios"
jest.mock("axios")
const mockedAxios = axios as jest.Mocked<typeof axios>

test("tokenFetcher", async () => {
  expect.assertions(2)
  let cred = { email: "test@example.com", password: "test" }
  // Non-string response data
  mockedAxios.post.mockResolvedValue({ status: httpStatusCodes.OK, data: 123 })
  try {
    await tokenFetcher(cred)
  } catch (e) {
    expect(e.message).toBe("unexpected response data: 123")
  }
  // Some random error
  mockedAxios.post.mockResolvedValue({
    status: httpStatusCodes.UNAUTHORIZED,
    data: "some random error",
  })
  try {
    await tokenFetcher(cred)
  } catch (e) {
    expect(e.message).toBe("some random error")
  }
})

test("string id user", async () => {
  expect.assertions(1)
  const user = {
    id: "1",
    email: "test@example.com",
    password_hash: "123",
    access: "Admin",
  }
  mockedAxios.get.mockResolvedValue({ status: httpStatusCodes.OK, data: user })
  tokenValidator("123").catch((e) =>
    expect(e.message).toStartWith("unexpected response data")
  )
})

test("null user", async () => {
  expect.assertions(1)
  const user = null
  mockedAxios.get.mockResolvedValue({ status: httpStatusCodes.OK, data: user })
  tokenValidator("123").catch((e) =>
    expect(e.message).toBe("unexpected response data: null")
  )
})

test("number email", async () => {
  expect.assertions(1)
  const user4 = {
    id: 1,
    email: 1,
    password_hash: "123",
    access: "Admin",
  }
  mockedAxios.get.mockResolvedValue({ status: httpStatusCodes.OK, data: user4 })
  tokenValidator("123").catch((e) =>
    expect(e.message).toStartWith("unexpected response data")
  )
})

test("number password hash", async () => {
  expect.assertions(1)
  const user = {
    id: 1,
    email: "test@example.com",
    password_hash: 1,
    access: "Admin",
  }
  mockedAxios.get.mockResolvedValue({ status: httpStatusCodes.OK, data: user })
  tokenValidator("123").catch((e) =>
    expect(e.message).toStartWith("unexpected response data")
  )
})

test("wrong access", async () => {
  expect.assertions(1)
  const user = {
    id: 1,
    email: "test@example.com",
    password_hash: "1",
    access: "Admin1",
  }
  mockedAxios.get.mockResolvedValue({ status: httpStatusCodes.OK, data: user })
  tokenValidator("123").catch((e) =>
    expect(e.message).toStartWith("unexpected response data")
  )
})

test("not enough fields", async () => {
  expect.assertions(1)
  const user = {
    id: 1,
    email: "test@example.com",
    password_hash: "1",
  }
  mockedAxios.get.mockResolvedValue({ status: httpStatusCodes.OK, data: user })
  tokenValidator("123").catch((e) =>
    expect(e.message).toStartWith("unexpected response data")
  )
})
