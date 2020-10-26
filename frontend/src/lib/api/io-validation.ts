import { pipe } from "fp-ts/function"
import { fold } from "fp-ts/Either"
import * as t from "io-ts"
import { PathReporter } from "io-ts/PathReporter"
import { TableMeta, TableData, TableRow } from "./project"

// https://github.com/gcanti/io-ts/issues/216#issuecomment-621588750
export function fromEnum<EnumType extends string>(
  enumName: string,
  theEnum: Record<string, EnumType>
): t.Type<EnumType, EnumType, unknown> {
  const isEnumValue = (input: unknown): input is EnumType =>
    Object.values<unknown>(theEnum).includes(input)

  return new t.Type<EnumType>(
    enumName,
    isEnumValue,
    (input, context) =>
      isEnumValue(input) ? t.success(input) : t.failure(input, context),
    t.identity
  )
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

export function decodeUserTable(meta: TableMeta, data: TableData) {
  return data.map((row) => {
    const decodedRow: TableRow = {}
    for (const [name, value] of Object.entries(row)) {
      const thisType = meta.cols.find((c) => c.name === name)?.postgres_type
      if (!thisType) {
        throw Error(`column ${name} not found in metadata`)
      }
      if (thisType === "timestamp with time zone") {
        decodedRow[name] = new Date(value)
      } else {
        decodedRow[name] = value
      }
    }
    return decodedRow
  })
}
