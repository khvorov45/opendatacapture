import React from "react"
import { AppBar, Toolbar, IconButton } from "@material-ui/core"
import BrightnessMediumIcon from "@material-ui/icons/BrightnessMedium"
import { createStyles, makeStyles, Theme } from "@material-ui/core/styles"

const useStyles = makeStyles((theme: Theme) =>
  createStyles({
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
        "font-size": "small",
      },
      "& .name": {
        "font-size": "1em",
      },
    },
  })
)

export default function Nav({
  handleThemeChange,
  currentProject,
}: {
  handleThemeChange: () => void
  currentProject?: string
}) {
  return (
    <AppBar position="sticky">
      <Toolbar>
        <ProjectInfo name={currentProject} />
        <ThemeSwitch handleThemeChange={handleThemeChange} />
      </Toolbar>
    </AppBar>
  )
}

function ProjectInfo({ name }: { name?: string }) {
  const classes = useStyles()
  if (!name) {
    return <></>
  }
  return (
    <div className={classes.projectInfo}>
      <div className="label">Project:</div>
      <div className="name">{name}</div>
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
