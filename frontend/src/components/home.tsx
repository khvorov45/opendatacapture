import React, { useState, useEffect, useCallback } from "react"
import { createStyles, makeStyles, Theme } from "@material-ui/core/styles"
import { Add, Send, DeleteForever } from "@material-ui/icons"
import { getUserProjects, Project, deleteProject } from "../lib/project"
import {
  IconButton,
  useTheme,
  TextField,
  FormHelperText,
  TableContainer,
  Table,
  TableHead,
  TableCell,
  TableRow,
  TableBody,
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
      <ProjectList
        token={token}
        projects={projects}
        onRemove={refreshProjectList}
      />
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
  projectRemover,
  onRemove,
}: {
  token: string
  projects: Project[]
  projectRemover?: (tok: string, name: string) => Promise<void>
  onRemove?: () => void
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

  // Functionality
  const projectRemoverResolved = projectRemover ?? deleteProject
  function removeProject(token: string, name: string) {
    projectRemoverResolved(token, name).then(() => onRemove?.())
  }
  return (
    <div className={classes.projectList} data-testid="project-list">
      {projects.length ? (
        <TableContainer>
          <Table>
            <TableHead>
              <TableRow>
                <TableCell>Name</TableCell>
                <TableCell></TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {projects.map((p) => (
                <ProjectEntry
                  key={p.user + p.name}
                  token={token}
                  project={p}
                  remover={removeProject}
                />
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      ) : (
        <NoProjects token={token} />
      )}
    </div>
  )
}

function ProjectEntry({
  token,
  project,
  remover,
}: {
  token: string
  project: Project
  remover: (tok: string, name: string) => void
}) {
  return (
    <TableRow>
      <TableCell align="center">{project.name}</TableCell>
      <TableCell>
        <ProjectRemoveButton onClick={() => remover(token, project.name)} />
      </TableCell>
    </TableRow>
  )
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
  return (
    <IconButton onClick={onClick}>
      <Add htmlColor={theme.palette.success.main} />
    </IconButton>
  )
}

function ProjectRemoveButton({ onClick }: { onClick?: () => void }) {
  const theme = useTheme()
  return (
    <IconButton onClick={onClick}>
      <DeleteForever htmlColor={theme.palette.error.main} />
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
