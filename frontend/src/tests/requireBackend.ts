/** All of these tests assume that the backend is running as if it was
 * started with the --clean option
 */

/* istanbul ignore file */

import { Access, LoginFailure, tokenFetcher, tokenValidator } from "../lib/auth"

test("token fetching and validating", async () => {
  expect.assertions(6)
  // Wrong token
  tokenValidator("123").catch((e) =>
    expect(e.message).toBe("Request failed with status code 401")
  )
  // Wrong password
  tokenFetcher({
    email: "admin@example.com",
    password: "123",
  }).catch((e) => expect(e.message).toBe(LoginFailure.WrongPassword))

  // Wrong email
  tokenFetcher({
    email: "user@example.com",
    password: "admin",
  }).catch((e) => expect(e.message).toBe(LoginFailure.EmailNotFound))

  // Correct credentials
  let token = await tokenFetcher({
    email: "admin@example.com",
    password: "admin",
  })
  let admin = await tokenValidator(token)
  expect(admin.access).toBe(Access.Admin)
  expect(admin.email).toBe("admin@example.com")
  expect(admin.id).toBe(1)
})
