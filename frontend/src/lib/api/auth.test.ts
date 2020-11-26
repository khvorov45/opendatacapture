// Test whatever can't be tested in fullstack
/* istanbul ignore file */
import httpStatusCodes from "http-status-codes"
import { fetchToken, validateToken } from "./auth"

import axios from "axios"
import {
  constructGet,
  constructPut,
  constructDelete,
  constructPost,
} from "../../tests/api"
jest.mock("axios")
const mockedAxios = axios as jest.Mocked<typeof axios>

const getreq = mockedAxios.get.mockImplementation(constructGet())
const putreq = mockedAxios.put.mockImplementation(constructPut())
const deletereq = mockedAxios.delete.mockImplementation(constructDelete())
const postreq = mockedAxios.post.mockImplementation(constructPost())
afterEach(() => {
  mockedAxios.get.mockImplementation(constructGet())
  mockedAxios.put.mockImplementation(constructPut())
  mockedAxios.delete.mockImplementation(constructDelete())
  mockedAxios.post.mockImplementation(constructPost())
})

test("fetchToken", async () => {
  expect.assertions(2)
  let cred = { email: "test@example.com", password: "test" }
  // Non-string response data
  mockedAxios.post.mockImplementation(
    constructPost({
      fetchToken: async () => ({ status: httpStatusCodes.OK, data: 123 }),
    })
  )
  try {
    await fetchToken(cred)
  } catch (e) {
    expect(e.message).toStartWith("decode error")
  }
  // Some random error
  mockedAxios.post.mockImplementation(
    constructPost({
      fetchToken: async () => ({
        status: httpStatusCodes.UNAUTHORIZED,
        data: "some random error",
      }),
    })
  )
  try {
    await fetchToken(cred)
  } catch (e) {
    expect(e.message).toBe("some random error")
  }
})
