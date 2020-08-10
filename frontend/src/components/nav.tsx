import React from "react"
import { AppBar, Toolbar, IconButton } from "@material-ui/core"
import BrightnessMediumIcon from "@material-ui/icons/BrightnessMedium"
import { createStyles, makeStyles, Theme } from "@material-ui/core/styles"

export default function Nav({
  handleThemeChange,
}: {
  handleThemeChange: () => void
}) {
  return (
    <AppBar position="sticky">
      <Toolbar>
        <ThemeSwitch handleThemeChange={handleThemeChange} />
      </Toolbar>
    </AppBar>
  )
}

function ThemeSwitch({ handleThemeChange }: { handleThemeChange: () => void }) {
  const useStyles = makeStyles((theme: Theme) =>
    createStyles({
      themeswitch: {
        "margin-left": "auto",
      },
    })
  )
  const classes = useStyles()
  return (
    <IconButton
      className={classes.themeswitch}
      data-testid="themeswitch"
      onClick={handleThemeChange}
    >
      <BrightnessMediumIcon />
    </IconButton>
  )
}
