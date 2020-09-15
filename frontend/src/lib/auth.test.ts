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

test("tokenValidator - bad users", async () => {
  expect.assertions(7)
  let tok = "123"

  const user2 = {
    id: "1",
    email: "test@example.com",
    password_hash: "123",
    access: "Admin",
  }
  mockedAxios.get.mockResolvedValue({ status: httpStatusCodes.OK, data: user2 })
  tokenValidator(tok).catch((e) =>
    expect(e.message).toStartWith("unexpected response data")
  )

  const user3 = null
  mockedAxios.get.mockResolvedValue({ status: httpStatusCodes.OK, data: user3 })
  tokenValidator(tok).catch((e) =>
    expect(e.message).toBe("unexpected response data: null")
  )

  const user4 = {
    id: 1,
    email: 1,
    password_hash: "123",
    access: "Admin",
  }
  mockedAxios.get.mockResolvedValue({ status: httpStatusCodes.OK, data: user4 })
  tokenValidator(tok).catch((e) =>
    expect(e.message).toStartWith("unexpected response data")
  )

  const user5 = {
    id: 1,
    email: "test@example.com",
    password_hash: 1,
    access: "Admin",
  }
  mockedAxios.get.mockResolvedValue({ status: httpStatusCodes.OK, data: user5 })
  tokenValidator(tok).catch((e) =>
    expect(e.message).toStartWith("unexpected response data")
  )

  const user6 = {
    id: 1,
    email: "test@example.com",
    password_hash: "1",
    access: "Admin1",
  }
  mockedAxios.get.mockResolvedValue({ status: httpStatusCodes.OK, data: user6 })
  tokenValidator(tok).catch((e) =>
    expect(e.message).toStartWith("unexpected response data")
  )

  const user7 = {
    id: 1,
    email: "test@example.com",
    password_hash: "1",
  }
  mockedAxios.get.mockResolvedValue({ status: httpStatusCodes.OK, data: user7 })
  tokenValidator(tok).catch((e) =>
    expect(e.message).toStartWith("unexpected response data")
  )

  const user8 = {
    id: 1,
    email: "test@example.com",
    password_hash: "1",
    access: "Admin",
    extra: "1",
  }
  mockedAxios.get.mockResolvedValue({ status: httpStatusCodes.OK, data: user8 })
  tokenValidator(tok).catch((e) =>
    expect(e.message).toStartWith("unexpected response data")
  )
})
