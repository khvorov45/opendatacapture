import React from "react"
import { createStyles, makeStyles, Theme } from "@material-ui/core/styles"

const useStyles = makeStyles((theme: Theme) =>
  createStyles({
    namedDivider: {
      display: "flex",
      alignItems: "center",
      textAlign: "center",
      color: theme.palette.text.secondary,
      "&::before, &::after": {
        content: "''",
        flex: 1,
        borderBottom: `1px solid ${theme.palette.divider}`,
      },
    },
  })
)

export function NamedDivider({
  className,
  name,
}: {
  className?: string
  name: string
}) {
  const classes = useStyles()
  return (
    <div
      className={`${classes.namedDivider}${className ? ` ${className}` : ""}`}
    >
      {name}
    </div>
  )
}
