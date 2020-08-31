import React, { useState, useEffect, useCallback } from "react"
import { createStyles, makeStyles, Theme } from "@material-ui/core/styles"
import { Add, Send, DeleteForever } from "@material-ui/icons"
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
  CircularProgress,
  Typography,
} from "@material-ui/core"
import { usePromiseTracker, trackPromise } from "react-promise-tracker"
import { getUserProjects, Project, deleteProject } from "../lib/project"
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
      {token ? <ProjectWidget token={token} /> : <CircularProgress />}
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
  const [projects, setProjects] = useState<Project[] | null>(null)
  const [errorMsg, setErrorMsg] = useState("")
  const refreshProjectList = useCallback(() => {
    setErrorMsg("")
    getUserProjects(token)
      .then((ps) => setProjects(ps))
      .catch((e) => setErrorMsg(e.message))
  }, [token])
  useEffect(() => {
    refreshProjectList()
  }, [refreshProjectList])
  return (
    <div className={classes.projectWidget} data-testid="projectWidget">
      {projects ? (
        <>
          <ProjectControl
            token={token}
            refreshProjectList={refreshProjectList}
            noProjects={projects.length === 0}
          />
          <ProjectList
            token={token}
            projects={projects}
            onRemove={refreshProjectList}
          />
        </>
      ) : (
        <CircularProgress />
      )}
      <FormHelperText error={true} data-testid="get-projects-error">
        {errorMsg}
      </FormHelperText>
    </div>
  )
}

function ProjectControl({
  token,
  refreshProjectList,
  noProjects,
}: {
  token: string
  refreshProjectList: () => void
  noProjects?: boolean
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
  const [formOpen, setFormOpen] = useState(noProjects ?? false)
  function handleCreate() {
    setFormOpen(!formOpen)
  }
  function handleProjectSubmit() {
    setFormOpen(false)
    refreshProjectList()
  }
  useEffect(() => {
    noProjects && setFormOpen(true)
  }, [noProjects])
  return (
    <div className={classes.projectControl} data-testid="projectControl">
      <ProjectControlButtons onCreate={handleCreate} />
      <ProjectCreateForm
        token={token}
        open={formOpen}
        onSuccess={handleProjectSubmit}
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
        justifyContent: "center",
        marginTop: "20px",
      },
    })
  )
  const classes = useStyles()
  return (
    <div className={classes.noProjects} data-testid="no-projects">
      <Typography color="textSecondary" variant="subtitle1">
        No projects found
      </Typography>
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
  onSuccess,
}: {
  token: string
  open: boolean
  projectCreator?: (token: string, name: string) => Promise<void>
  onSuccess?: () => void
}) {
  // Appearance
  const useStyles = makeStyles((theme: Theme) =>
    createStyles({
      createForm: {
        visibility: open ? "visible" : "hidden",
        height: open ? "auto" : "0px",
        display: "flex",
        alignItems: "center",
        flexDirection: "column",
      },
    })
  )
  const classes = useStyles()

  // Functionality
  const [name, setName] = useState("")
  const [errorMsg, setErrorMsg] = useState("")
  const { promiseInProgress } = usePromiseTracker({ area: "create-project" })

  const projectCreatorResolved = projectCreator ?? createProject
  function handleSubmit() {
    trackPromise(projectCreatorResolved(token, name), "create-project")
      .then(() => {
        onSuccess?.()
        setName("")
        setErrorMsg("")
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
      <div>
        <TextField
          data-testid="project-name-field"
          label="New project name..."
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
        {promiseInProgress ? (
          <CircularProgress />
        ) : (
          <IconButton
            onClick={(e) => {
              e.preventDefault()
              handleSubmit()
            }}
            data-testid="login-submit"
            type="submit"
            disabled={name.length === 0}
          >
            <Send />
          </IconButton>
        )}
      </div>
      <FormHelperText error={true} data-testid="create-project-error">
        {errorMsg}
      </FormHelperText>
    </form>
  )
}
