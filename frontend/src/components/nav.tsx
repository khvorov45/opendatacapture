import React from "react"
import { AppBar, Toolbar, IconButton } from "@material-ui/core"
import BrightnessMediumIcon from "@material-ui/icons/BrightnessMedium"

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
  return (
    <IconButton data-testid="themeswitch" onClick={handleThemeChange}>
      <BrightnessMediumIcon />
    </IconButton>
  )
}
