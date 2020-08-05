import React from "react"
import { AppBar, Toolbar, IconButton } from "@material-ui/core"
import BrightnessMediumIcon from "@material-ui/icons/BrightnessMedium"

export default function Nav({
  darkState,
  handleThemeChange,
}: {
  darkState: boolean
  handleThemeChange: () => void
}) {
  return (
    <AppBar>
      <Toolbar>
        <IconButton data-testid="themeswitch" onClick={handleThemeChange}>
          <BrightnessMediumIcon />
        </IconButton>
      </Toolbar>
    </AppBar>
  )
}
