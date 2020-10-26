/* istanbul ignore file */

import { decodeUserTable, fromEnum } from "./io-validation"
import { Access } from "./auth"
import { isLeft, right } from "fp-ts/lib/Either"
import * as t from "io-ts"
import { decode } from "./io-validation"
import { table1, table1data } from "../../tests/util"
import { TableData } from "./project"

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

test("decodeUserTable", () => {
  // Y NO CLONE METHOD ON ARRAYS WTF JAVASCRIPT
  let table1dataDecoded: TableData = JSON.parse(JSON.stringify(table1data))
  table1dataDecoded.map((row) => {
    row.dob = new Date(row.dob)
    return row
  })
  expect(table1data).not.toEqual(table1dataDecoded)
  expect(decodeUserTable(table1, table1data)).toEqual(table1dataDecoded)
})

test("decodeUserTable with extra column in data", () => {
  expect.assertions(1)
  let table1dataWrong = JSON.parse(JSON.stringify(table1data))
  table1dataWrong[0].extraCol = ""
  try {
    decodeUserTable(table1, table1dataWrong)
  } catch (e) {
    expect(e.message).toBe("column extraCol not found in metadata")
  }
})
