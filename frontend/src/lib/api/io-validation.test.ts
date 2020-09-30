/* istanbul ignore file */

import { fromEnum } from "./io-validation"
import { Access } from "./auth"
import { isLeft, right } from "fp-ts/lib/Either"
import * as t from "io-ts"
import { decode } from "./io-validation"

test("enum validation", () => {
  const AccessV = fromEnum<Access>("Access", Access)
  expect(AccessV.is("User")).toBeTrue()
  expect(AccessV.is("Admin")).toBeTrue()
  expect(AccessV.is("admin")).toBeFalse()
  expect(AccessV.decode("Admin")).toEqual(right("Admin"))
  expect(isLeft(AccessV.decode("admin"))).toBeTrue()
  expect(AccessV.validate("Admin", [])).toEqual(right("Admin"))
  expect(isLeft(AccessV.validate("admin", []))).toBeTrue()
})

test("decode correct", async () => {
  const TypeV = t.type({
    id: t.number,
    email: t.string,
  })
  const correct = {
    id: 1,
    email: "1",
  }
  expect(await decode(TypeV, correct)).toEqual(correct)
})

test("decode wrong", () => {
  expect.assertions(1)
  const TypeV = t.type({
    id: t.number,
    email: t.string,
  })
  const wrong = {
    id: 1,
    email: 1,
  }
  decode(TypeV, wrong).catch((e) =>
    expect(e.message).toStartWith("decode error: ")
  )
})
