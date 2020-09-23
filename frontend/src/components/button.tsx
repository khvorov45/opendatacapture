import React, { ReactNode } from "react"
import { Link } from "react-router-dom"
import { createStyles, makeStyles, Theme } from "@material-ui/core/styles"
import {
  Button,
  CircularProgress,
  FormHelperText,
  IconButton,
  useTheme,
} from "@material-ui/core"
import Add from "@material-ui/icons/Add"
import Refresh from "@material-ui/icons/Refresh"
import DeleteForever from "@material-ui/icons/DeleteForever"

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
    link: {
      textTransform: "none",
      "&.active": {
        backgroundColor: "var(--palette-bg-highlight)",
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

export function ButtonLink({
  children,
  active,
  to,
}: {
  children: ReactNode
  active: boolean
  to: string
}) {
  const classes = useStyles()
  return (
    <Button
      className={`${classes.link}${active ? " active" : ""}`}
      component={Link}
      to={to}
    >
      {children}
    </Button>
  )
}

export function RefreshButton({
  onClick,
  dataTestId,
  inProgress,
}: {
  onClick?: () => void
  dataTestId?: string
  inProgress?: boolean
}) {
  if (inProgress) {
    return <CircularProgress />
  }
  return (
    <IconButton onClick={onClick} data-testid={dataTestId}>
      <Refresh />
    </IconButton>
  )
}

export function DeleteButton({ onClick }: { onClick: () => void }) {
  const theme = useTheme()
  return (
    <IconButton onClick={(e) => onClick()}>
      <DeleteForever htmlColor={theme.palette.error.main} />
    </IconButton>
  )
}
