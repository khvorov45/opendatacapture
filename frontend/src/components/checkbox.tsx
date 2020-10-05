import {
  FormControlLabel,
  Checkbox as MaterialCheckbox,
} from "@material-ui/core"
import React, { ChangeEvent } from "react"

export default function Checkbox({
  checked,
  onChange,
  label,
  hidden,
  readOnly,
  dataTestId,
}: {
  checked: boolean
  onChange: (value: boolean) => void
  label: string
  hidden?: boolean
  readOnly?: boolean
  dataTestId?: string
}) {
  return (
    <FormControlLabel
      className={`${hidden ? "hidden" : ""}`}
      control={
        <MaterialCheckbox
          checked={checked}
          onChange={(e: ChangeEvent<HTMLInputElement>) =>
            onChange(e.target.checked)
          }
          disabled={readOnly}
          data-testid={dataTestId}
        />
      }
      label={label}
    />
  )
}
