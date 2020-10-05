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
      borderRadius: 0,
      "&.active": {
        backgroundColor: "var(--palette-bg-highlight)",
      },
    },
    buttonContainer: {
      width: 48,
      height: 48,
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

export function IconButtonWithProgress({
  children,
  onClick,
  dataTestId,
  inProgress,
  disabled,
}: {
  children: ReactNode
  onClick?: () => void
  dataTestId?: string
  inProgress?: boolean
  disabled?: boolean
}) {
  const classes = useStyles()
  if (inProgress) {
    return (
      <div className={classes.buttonContainer}>
        <CircularProgress />
      </div>
    )
  }
  return (
    <IconButton onClick={onClick} data-testid={dataTestId} disabled={disabled}>
      {children}
    </IconButton>
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
  className,
  dataTestId,
  onClick,
}: {
  children: ReactNode
  active: boolean
  to: string
  className?: string
  dataTestId?: string
  onClick?: () => void
}) {
  const classes = useStyles()
  return (
    <Button
      data-testid={dataTestId}
      className={`${classes.link}${active ? " active" : ""}${
        className ? " " + className : ""
      }`}
      component={Link}
      to={to}
      onClick={onClick}
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
  return (
    <IconButtonWithProgress
      onClick={onClick}
      dataTestId={dataTestId}
      inProgress={inProgress}
    >
      <Refresh />
    </IconButtonWithProgress>
  )
}

export function DeleteButton({
  onClick,
  dataTestId,
  inProgress,
}: {
  onClick: () => void
  dataTestId?: string
  inProgress?: boolean
}) {
  const theme = useTheme()
  return (
    <IconButtonWithProgress
      dataTestId={dataTestId}
      onClick={onClick}
      inProgress={inProgress}
    >
      <DeleteForever htmlColor={theme.palette.error.main} />
    </IconButtonWithProgress>
  )
}
