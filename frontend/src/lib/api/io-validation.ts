import { pipe } from "fp-ts/function"
import { fold } from "fp-ts/Either"
import * as t from "io-ts"
import { PathReporter } from "io-ts/PathReporter"

export class EnumType<A> extends t.Type<A> {
  public readonly _tag: "EnumType" = "EnumType"
  public enumObject!: object
  public constructor(e: object, name?: string) {
    super(
      name || "enum",
      (u): u is A => {
        if (!Object.values(this.enumObject).find((v) => v === u)) {
          return false
        }
        // enum reverse mapping check
        if (typeof (this.enumObject as any)[u as string] === "number") {
          return false
        }

        return true
      },
      (u, c) => (this.is(u) ? t.success(u) : t.failure(u, c)),
      t.identity
    )
    this.enumObject = e
  }
}

export function createEnumType<T>(e: object, name?: string) {
  return new EnumType<T>(e, name)
}

export async function decode<T, O, I>(
  validator: t.Type<T, O, I>,
  input: I
): Promise<T> {
  const result = validator.decode(input)
  return pipe(
    result,
    fold(
      (errors) => {
        throw Error("decode error: " + PathReporter.report(result))
      },
      (value) => value
    )
  )
}
