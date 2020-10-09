import {
  makeStyles,
  Theme,
  createStyles,
  FormControl,
  InputLabel,
  Select as MaterialSelect,
} from "@material-ui/core"
import React, { ReactNode } from "react"

const useStyles = makeStyles((theme: Theme) =>
  createStyles({
    select: {
      minWidth: 80,
    },
  })
)

export default function Select({
  children,
  value,
  onChange,
  id,
  label,
  hidden,
  readOnly,
  dataTestId,
}: {
  children: ReactNode
  value: string
  onChange?: (value: string) => void
  id: string
  label: string
  hidden?: boolean
  readOnly?: boolean
  dataTestId?: string
}) {
  const classes = useStyles()
  return (
    <FormControl className={`${classes.select}${hidden ? " hidden" : ""}`}>
      <InputLabel id={id + "-select-label"}>{label}</InputLabel>
      <MaterialSelect
        data-testid={dataTestId}
        labelId={id + "-select-label"}
        id={id}
        value={value}
        onChange={(e) => onChange?.(e.target.value as string)}
        disabled={readOnly}
      >
        {children}
      </MaterialSelect>
    </FormControl>
  )
}
