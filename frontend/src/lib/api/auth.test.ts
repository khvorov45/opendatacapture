// Test whatever can't be tested in fullstack
/* istanbul ignore file */
import httpStatusCodes from "http-status-codes"
import axios from "axios"
import { fetchToken } from "./auth"
import { constructPost } from "../../tests/api"

jest.mock("axios")
const mockedAxios = axios as jest.Mocked<typeof axios>

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
