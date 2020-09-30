import { fromEnum } from "./io-validation"
import { Access } from "./auth"

test("createEnumType", () => {
  const AccessV = fromEnum<Access>("Access", Access)
  expect(AccessV.is("User")).toBeTrue()
  expect(AccessV.is("Admin")).toBeTrue()
  expect(AccessV.is("admin")).toBeFalse()
})
