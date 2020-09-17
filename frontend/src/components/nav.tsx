import React from "react"
import { AppBar, Toolbar, IconButton } from "@material-ui/core"
import BrightnessMediumIcon from "@material-ui/icons/BrightnessMedium"
import { createStyles, makeStyles, Theme } from "@material-ui/core/styles"
import { useLocation } from "react-router-dom"

const useStyles = makeStyles((theme: Theme) =>
  createStyles({
    nav: {
      overflow: "auto",
      "& .toolbar": {
        display: "flex",
      },
    },
    themeswitch: {
      marginLeft: "auto",
    },
    projectInfo: {
      display: "flex",
      flexDirection: "column",
      "& *": {
        margin: "auto",
      },
      "& .label": {
        "font-size": "0.9em",
      },
      "& .name": {
        "font-size": "1.1em",
      },
    },
  })
)

export default function Nav({
  handleThemeChange,
}: {
  handleThemeChange: () => void
}) {
  const classes = useStyles()

  return (
    <AppBar position="relative" className={classes.nav}>
      <Toolbar className="toolbar">
        <ProjectInfo />
        <ThemeSwitch handleThemeChange={handleThemeChange} />
      </Toolbar>
    </AppBar>
  )
}

function ProjectInfo() {
  const location = useLocation()
  const classes = useStyles()
  if (!location.pathname.startsWith("/project")) {
    return <></>
  }
  return (
    <div className={classes.projectInfo}>
      <div className="label">Project</div>
      <div className="name">
        {location.pathname.match(/^\/project\/([^/]*)/)?.[1]}
      </div>
    </div>
  )
}

function ThemeSwitch({ handleThemeChange }: { handleThemeChange: () => void }) {
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
