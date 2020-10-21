import React from "react"
import { Button, IconButton } from "@material-ui/core"
import BrightnessMediumIcon from "@material-ui/icons/BrightnessMedium"
import PowerSettingsNewIcon from "@material-ui/icons/PowerSettingsNew"
import { createStyles, makeStyles, Theme } from "@material-ui/core/styles"
import { useLocation, useRouteMatch } from "react-router-dom"
import { ButtonLink } from "./button"
import toProperCase from "../lib/to-proper-case"

const useStyles = makeStyles((theme: Theme) =>
  createStyles({
    nav: {
      overflow: "auto",
      display: "flex",
      justifyContent: "space-between",
      alignItems: "center",
      backgroundColor: "var(--palette-bg-alt)",
      borderBottom: `1px solid ${theme.palette.divider}`,
    },
    projectInfo: {
      display: "flex",
      flexDirection: "column",
      "& *": {
        margin: "auto",
      },
      "& .label": {
        alignSelf: "flex-end",
        "font-size": "0.9em",
      },
      "& .name": {
        "font-size": "1.1em",
      },
    },
    simpleNav: {
      backgroundColor: "var(--palette-bg-alt)",
      borderBottom: `1px solid ${theme.palette.divider}`,
    },
    link: {
      height: 48,
    },
  })
)

export default function Nav({
  handleThemeChange,
  onLogout,
}: {
  handleThemeChange: () => void
  onLogout: () => void
}) {
  const location = useLocation()
  const classes = useStyles()
  const loginHide = location.pathname.startsWith("/login") ? "nodisplay" : ""
  return (
    <div className={classes.nav}>
      <div>
        <ButtonLink
          dataTestId="home-link"
          className={`${classes.link} ${loginHide}`}
          active={location.pathname === "/"}
          to="/"
        >
          Projects
        </ButtonLink>
      </div>
      <div>
        <ProjectInfo />
      </div>
      <div>
        <IconButton
          onClick={onLogout}
          className={loginHide}
          data-testid="logout-button"
        >
          <PowerSettingsNewIcon />
        </IconButton>
        <ThemeSwitch handleThemeChange={handleThemeChange} />
      </div>
    </div>
  )
}

function ProjectInfo() {
  const location = useLocation()
  const classes = useStyles()
  const show = location.pathname.startsWith("/project")
  return (
    <div
      className={`${classes.projectInfo}${show ? "" : " nodisplay"}`}
      data-testid="nav-project-info"
    >
      <span className="label">Project</span>
      <span className="name">
        {location.pathname.match(/^\/project\/([^/]*)/)?.[1]}
      </span>
    </div>
  )
}

function ThemeSwitch({ handleThemeChange }: { handleThemeChange: () => void }) {
  return (
    <IconButton data-testid="themeswitch" onClick={handleThemeChange}>
      <BrightnessMediumIcon />
    </IconButton>
  )
}

export function SimpleNav({
  links,
  dataTestId,
  onClick,
  active,
}: {
  links: string[]
  dataTestId?: string
  onClick?: (l: string) => void
  active?: (l: string) => boolean
}) {
  const { url } = useRouteMatch()
  const { pathname } = useLocation()
  const classes = useStyles()
  return (
    <div className={classes.simpleNav} data-testid={dataTestId}>
      {links.map((l) => (
        <ButtonLink
          key={l}
          className={classes.link}
          active={active ? active(l) : pathname.includes(l)}
          to={`${url}/${l}`}
          onClick={() => onClick?.(l)}
        >
          {toProperCase(l)}
        </ButtonLink>
      ))}
    </div>
  )
}
