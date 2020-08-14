/** All of these tests assume that the backend is running as if it was
 * started with the --clean option
 */

import { tokenValidator } from "../lib/auth"

test("tokenValidator", async () => {
  tokenValidator("123").catch((e) =>
    expect(e.message).toBe("Request failed with status code 401")
  )
})
