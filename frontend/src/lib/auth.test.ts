/* istanbul ignore file */
import { tokenFetcher, tokenValidator } from "./auth"

import axios from "axios"
jest.mock("axios")
const mockedAxios = axios as jest.Mocked<typeof axios>

test("tokenFetcher", async () => {
  expect.assertions(6)
  let cred = { email: "test@example.com", password: "test" }
  mockedAxios.post.mockResolvedValue({ status: 200, data: "123" })
  expect(await tokenFetcher(cred)).toBe("123")
  mockedAxios.post.mockResolvedValue({ status: 500, data: "123" })
  tokenFetcher(cred).catch((e) =>
    expect(e.message).toBe('login failed with status 500 and data "123"')
  )
  mockedAxios.post.mockResolvedValue({ status: 200, data: null })
  tokenFetcher(cred).catch((e) =>
    expect(e.message).toBe("unexpected response data: null")
  )
  mockedAxios.post.mockResolvedValue({ status: 201, data: "NoSuchUserEmail" })
  tokenFetcher(cred).catch((e) => expect(e.message).toBe("EmailNotFound"))
  mockedAxios.post.mockResolvedValue({ status: 201, data: "WrongPassword" })
  tokenFetcher(cred).catch((e) => expect(e.message).toBe("WrongPassword"))
  mockedAxios.post.mockImplementation(async (address, data, config) => {
    if (!config) throw Error()
    if (!config.validateStatus) throw Error()
    if (config.validateStatus(201)) throw Error()
    return { status: 200, data: "123" }
  })
  expect(await tokenFetcher(cred)).toBe("123")
})

test("tokenValidator", async () => {
  expect.assertions(8)
  let tok = "123"
  let user = {
    id: 1,
    email: "test@example.com",
    password_hash: "123",
    access: "Admin",
  }
  mockedAxios.get.mockResolvedValue({ data: user })
  expect(await tokenValidator(tok)).toBe(user)
  const user2 = {
    id: "1",
    email: "test@example.com",
    password_hash: "123",
    access: "Admin",
  }
  mockedAxios.get.mockResolvedValue({ data: user2 })
  tokenValidator(tok).catch((e) =>
    expect(e.message).toStartWith("unexpected response data")
  )
  const user3 = null
  mockedAxios.get.mockResolvedValue({ data: user3 })
  tokenValidator(tok).catch((e) =>
    expect(e.message).toBe("unexpected response data: null")
  )
  const user4 = {
    id: 1,
    email: 1,
    password_hash: "123",
    access: "Admin",
  }
  mockedAxios.get.mockResolvedValue({ data: user4 })
  tokenValidator(tok).catch((e) =>
    expect(e.message).toStartWith("unexpected response data")
  )
  const user5 = {
    id: 1,
    email: "test@example.com",
    password_hash: 1,
    access: "Admin",
  }
  mockedAxios.get.mockResolvedValue({ data: user5 })
  tokenValidator(tok).catch((e) =>
    expect(e.message).toStartWith("unexpected response data")
  )
  const user6 = {
    id: 1,
    email: "test@example.com",
    password_hash: "1",
    access: "Admin1",
  }
  mockedAxios.get.mockResolvedValue({ data: user6 })
  tokenValidator(tok).catch((e) =>
    expect(e.message).toStartWith("unexpected response data")
  )
  const user7 = {
    id: 1,
    email: "test@example.com",
    password_hash: "1",
  }
  mockedAxios.get.mockResolvedValue({ data: user7 })
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
  mockedAxios.get.mockResolvedValue({ data: user8 })
  tokenValidator(tok).catch((e) =>
    expect(e.message).toStartWith("unexpected response data")
  )
})
