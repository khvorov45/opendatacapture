import React, { useState, useEffect, useCallback } from "react"
import { createStyles, makeStyles, Theme } from "@material-ui/core/styles"
import Add from "@material-ui/icons/Add"
import Send from "@material-ui/icons/Send"
import { getUserProjects, Project } from "../lib/project"
import {
  IconButton,
  useTheme,
  TextField,
  FormHelperText,
} from "@material-ui/core"
import { createProject } from "../lib/project"

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
  const [projects, setProjects] = useState<Project[]>([])
  const [errorMsg, setErrorMsg] = useState("")
  const refreshProjectList = useCallback(
    () =>
      getUserProjects(token)
        .then((ps) => setProjects(ps))
        .catch((e) => setErrorMsg(e.message)),
    [token]
  )
  useEffect(() => {
    refreshProjectList()
  }, [refreshProjectList])
  return (
    <div className={classes.projectWidget} data-testid="projectWidget">
      <ProjectControl token={token} refreshProjectList={refreshProjectList} />
      <ProjectList token={token} projects={projects} />
      <FormHelperText error={true} data-testid="get-projects-error">
        {errorMsg}
      </FormHelperText>
    </div>
  )
}

function ProjectControl({
  token,
  refreshProjectList,
}: {
  token: string
  refreshProjectList: () => Promise<void>
}) {
  const useStyles = makeStyles((theme: Theme) =>
    createStyles({
      projectControl: {
        display: "flex",
        alignItems: "flex-start",
        flexDirection: "column",
      },
    })
  )
  const classes = useStyles()
  const [formOpen, setFormOpen] = useState(false)
  function handleCreate() {
    setFormOpen(!formOpen)
  }
  function handleProjectSubmit() {
    refreshProjectList().then(() => setFormOpen(false))
  }
  return (
    <div className={classes.projectControl} data-testid="projectControl">
      <ProjectControlButtons onCreate={handleCreate} />
      <ProjectCreateForm
        token={token}
        open={formOpen}
        onSubmit={handleProjectSubmit}
      />
    </div>
  )
}

function ProjectControlButtons({ onCreate }: { onCreate?: () => void }) {
  const useStyles = makeStyles((theme: Theme) =>
    createStyles({
      projectControlButtons: {
        display: "flex",
        alignItems: "flex-start",
      },
    })
  )
  const classes = useStyles()
  return (
    <div
      className={classes.projectControlButtons}
      data-testid="project-control-buttons"
    >
      <ProjectCreateButton onClick={onCreate} />
    </div>
  )
}

function ProjectList({
  token,
  projects,
}: {
  token: string
  projects: Project[]
}) {
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
  return (
    <div className={classes.projectList} data-testid="project-list">
      {projects.length ? (
        projects.map((p) => (
          <ProjectEntry key={p.user + p.name} project={p} token={token} />
        ))
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
      <p>Click</p> <ProjectCreateButton /> <p>to create a project</p>
    </div>
  )
}

function ProjectCreateButton({ onClick }: { onClick?: () => void }) {
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
    <IconButton className={classes.create} onClick={onClick}>
      <Add htmlColor={theme.palette.success.main} />
    </IconButton>
  )
}

function ProjectCreateForm({
  token,
  open,
  projectCreator,
  onSubmit,
}: {
  token: string
  open: boolean
  projectCreator?: (token: string, name: string) => Promise<void>
  onSubmit?: () => void
}) {
  // Appearance
  const useStyles = makeStyles((theme: Theme) =>
    createStyles({
      createForm: {
        display: open ? "flex" : "none",
        alignItems: "center",
      },
    })
  )
  const classes = useStyles()

  // Functionality
  const [name, setName] = useState("")
  const [errorMsg, setErrorMsg] = useState("")

  const projectCreatorResolved = projectCreator ?? createProject
  function handleSubmit() {
    projectCreatorResolved(token, name)
      .then(() => {
        setName("")
        setErrorMsg("")
        onSubmit?.()
      })
      .catch((e) => {
        if (e.message.startsWith("ProjectAlreadyExists")) {
          setErrorMsg(`Name '${name}' already in use`)
        } else {
          setErrorMsg(e.message)
        }
      })
  }
  return (
    <form className={classes.createForm} data-testid="project-create-form">
      <TextField
        data-testid="project-name-field"
        label="Name"
        value={name}
        onChange={(e) => setName(e.target.value)}
      />
      <IconButton
        onClick={(e) => {
          e.preventDefault()
          handleSubmit()
        }}
        data-testid="login-submit"
        type="submit"
      >
        <Send />
      </IconButton>
      <FormHelperText error={true} data-testid="create-project-error">
        {errorMsg}
      </FormHelperText>
    </form>
  )
}
