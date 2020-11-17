// Test whatever can't be tested in fullstack
/* istanbul ignore file */

import axios from "axios"
import httpStatusCodes from "http-status-codes"
import { constructPut } from "../../tests/api"
import { newUserCred } from "../../tests/data"
import { createUser } from "./user"
jest.mock("axios")
const mockedAxios = axios as jest.Mocked<typeof axios>

mockedAxios.put.mockImplementation(constructPut())

test("createUser", async () => {
  mockedAxios.put.mockResolvedValueOnce({
    status: httpStatusCodes.INTERNAL_SERVER_ERROR,
    data: "some create user error",
  })
  expect.assertions(1)
  try {
    await createUser(newUserCred)
  } catch (e) {
    expect(e.message).toBe("some create user error")
  }
})
