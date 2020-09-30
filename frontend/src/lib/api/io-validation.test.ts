import { createEnumType } from "./io-validation"
import { Access } from "./auth"

test("createEnumType", () => {
  const AccessV = createEnumType<Access>(Access, "Access")
  expect(AccessV.is("User")).toBeTrue()
  expect(AccessV.is("Admin")).toBeTrue()
  expect(AccessV.is("admin")).toBeFalse()
})
