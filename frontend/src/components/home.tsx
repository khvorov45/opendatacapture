import React, { useEffect, useState } from "react"
import { createStyles, makeStyles, Theme } from "@material-ui/core/styles"
import Add from "@material-ui/icons/Add"
import { getUserProjects, Project } from "../lib/project"
import { IconButton, useTheme } from "@material-ui/core"

export default function Home({ token }: { token: string | null }) {
  const useStyles = makeStyles((theme: Theme) =>
    createStyles({
      homePage: {
        display: "flex",
        justifyContent: "center",
      },
    })
  )
  const classes = useStyles()
  return (
    <div className={classes.homePage} data-testid="homepage">
      {token ? <ProjectWidget token={token} /> : "Loading"}
    </div>
  )
}

function ProjectWidget({ token }: { token: string }) {
  const useStyles = makeStyles((theme: Theme) =>
    createStyles({
      projectWidget: {
        display: "flex",
        justifyContent: "center",
        flexDirection: "column",
        marginTop: "20px",
      },
    })
  )
  const classes = useStyles()
  return (
    <div className={classes.projectWidget} data-testid="projectWidget">
      <ProjectControl token={token} />
      <ProjectList token={token} />
    </div>
  )
}

function ProjectControl({ token }: { token: string }) {
  const useStyles = makeStyles((theme: Theme) =>
    createStyles({
      projectControl: {
        display: "flex",
        alignItems: "center",
      },
    })
  )
  const classes = useStyles()
  return (
    <div className={classes.projectControl} data-testid="projectControl">
      <ProjectCreate token={token} />
    </div>
  )
}

function ProjectList({ token }: { token: string }) {
  const useStyles = makeStyles((theme: Theme) =>
    createStyles({
      projectList: {
        display: "flex",
        justifyContent: "center",
        flexDirection: "column",
      },
    })
  )
  const classes = useStyles()
  let [projects, setProjects] = useState<Project[]>([])
  useEffect(() => {
    getUserProjects(token).then((ps) => setProjects(ps))
  }, [token])
  return (
    <div className={classes.projectList} data-testid="project-list">
      {projects.length ? (
        projects.map((p) => <ProjectEntry project={p} token={token} />)
      ) : (
        <NoProjects token={token} />
      )}
    </div>
  )
}

function ProjectEntry({ project, token }: { project: Project; token: string }) {
  return <div>Project entry {JSON.stringify(project)}</div>
}

function NoProjects({ token }: { token: string }) {
  const useStyles = makeStyles((theme: Theme) =>
    createStyles({
      noProjects: {
        display: "flex",
        alignItems: "center",
      },
    })
  )
  const classes = useStyles()
  return (
    <div className={classes.noProjects} data-testid="no-projects">
      <p>Click</p> <ProjectCreate token={token} /> <p>to create a project</p>
    </div>
  )
}

function ProjectCreate({ token }: { token: string }) {
  const theme = useTheme()
  const useStyles = makeStyles((theme: Theme) =>
    createStyles({
      create: {
        cursor: "pointer",
      },
    })
  )
  const classes = useStyles()
  return (
    <IconButton className={classes.create} onClick={() => console.log("click")}>
      <Add htmlColor={theme.palette.success.main} />
    </IconButton>
  )
}
