import React, { ReactNode } from "react"
import { createStyles, makeStyles, Theme } from "@material-ui/core/styles"
import { FormHelperText } from "@material-ui/core"

const useStyles = makeStyles((theme: Theme) =>
  createStyles({
    buttonArray: {
      display: "flex",
      flexDirection: "column",
      "& div.buttons": {
        display: "flex",
        alignItems: "center",
      },
    },
  })
)

export default function ButtonArray({
  errorMsg,
  children,
  errorTestId,
}: {
  errorMsg?: string
  children: ReactNode
  errorTestId?: string
}) {
  const classes = useStyles()
  return (
    <div className={classes.buttonArray}>
      <div className="buttons">{children}</div>
      <FormHelperText error={true} data-testid={errorTestId}>
        {errorMsg}
      </FormHelperText>
    </div>
  )
}
