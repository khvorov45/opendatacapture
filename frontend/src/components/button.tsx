import React, { ReactNode } from "react"
import { createStyles, makeStyles, Theme } from "@material-ui/core/styles"
import { FormHelperText, IconButton, useTheme } from "@material-ui/core"
import Add from "@material-ui/icons/Add"

const useStyles = makeStyles((theme: Theme) =>
  createStyles({
    buttonArray: {
      display: "flex",
      flexDirection: "column",
      "& div.buttons": {
        display: "flex",
        alignItems: "center",
        "&.center": {
          justifyContent: "center",
        },
      },
    },
  })
)

export function ButtonArray({
  errorMsg,
  children,
  errorTestId,
  className,
  center,
}: {
  errorMsg?: string
  children: ReactNode
  errorTestId?: string
  className?: string
  center?: boolean
}) {
  const classes = useStyles()
  return (
    <div
      className={`${classes.buttonArray}${className ? ` ${className}` : ""}`}
    >
      <div className={`buttons${center ? " center" : ""}`}>{children}</div>
      <FormHelperText error={true} data-testid={errorTestId}>
        {errorMsg}
      </FormHelperText>
    </div>
  )
}

export function CreateButton({
  onClick,
  dataTestId,
}: {
  onClick?: () => void
  dataTestId?: string
}) {
  const theme = useTheme()
  return (
    <IconButton onClick={onClick} data-testid={dataTestId}>
      <Add htmlColor={theme.palette.success.main} />
    </IconButton>
  )
}