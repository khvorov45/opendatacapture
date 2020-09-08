import React, { useState, useEffect, useCallback } from "react"
import { createStyles, makeStyles, Theme } from "@material-ui/core/styles"
import Add from "@material-ui/icons/Add"
import Send from "@material-ui/icons/Send"
import DeleteForever from "@material-ui/icons/DeleteForever"
import Refresh from "@material-ui/icons/Refresh"
import {
  IconButton,
  useTheme,
  TextField,
  FormHelperText,
  TableContainer,
  Table,
  TableHead,
  TableBody,
  CircularProgress,
  Typography,
} from "@material-ui/core"
import { usePromiseTracker, trackPromise } from "react-promise-tracker"
import {
  getUserProjects,
  Project,
  deleteProject,
  createProject,
} from "../lib/project"
import { StyledTableRow, StyledTableCell } from "./table"

const useStyles = makeStyles((theme: Theme) =>
  createStyles({
    homePage: {
      display: "flex",
      justifyContent: "center",
    },
    projectWidget: {
      display: "flex",
      justifyContent: "center",
      flexDirection: "column",
      marginTop: "20px",
      border: `1px solid ${theme.palette.divider}`,
    },
    projectControl: {
      display: "flex",
      alignItems: "flex-start",
      flexDirection: "column",
    },
    projectControlButtons: {
      display: "flex",
      alignItems: "flex-start",
    },
    projectList: {
      display: "flex",
      justifyContent: "center",
      flexDirection: "column",
    },
    noProjects: {
      display: "flex",
      alignItems: "center",
      justifyContent: "center",
      marginTop: "20px",
    },
    projectEntry: {
      "&.hidden": {
        display: "none",
      },
    },
    createForm: {
      visibility: "visible",
      overflow: "hidden",
      maxHeight: "100px",
      display: "flex",
      alignItems: "center",
      flexDirection: "column",
      transition: "max-height 0.1s",
      paddingLeft: "5px",
      "&.hidden": {
        visibility: "hidden",
        maxHeight: "0px",
      },
    },
    centered: {
      display: "flex",
      justifyContent: "center",
    },
  })
)

export default function Home({ token }: { token: string | null }) {
  const classes = useStyles()
  return (
    <div className={classes.homePage} data-testid="homepage">
      {token ? <ProjectWidget token={token} /> : <CircularProgress />}
    </div>
  )
}

export function ProjectWidget({ token }: { token: string }) {
  const classes = useStyles()
  const [projects, setProjects] = useState<Project[] | null>(null)
  const [errorMsg, setErrorMsg] = useState("")
  const refreshProjectList = useCallback(() => {
    trackPromise(getUserProjects(token), "get-projects")
      .then((ps) => {
        setErrorMsg("")
        setProjects(ps)
      })
      .catch((e) => setErrorMsg(e.message))
  }, [token])
  useEffect(() => {
    refreshProjectList()
  }, [refreshProjectList])
  return (
    <div className={classes.projectWidget} data-testid="project-widget">
      <ProjectControl
        token={token}
        refreshProjectList={refreshProjectList}
        noProjects={projects ? projects.length === 0 : false}
      />
      {projects ? (
        <ProjectList
          token={token}
          projects={projects}
          onRemove={refreshProjectList}
        />
      ) : errorMsg ? (
        <></>
      ) : (
        <div className={classes.centered}>
          <CircularProgress />
        </div>
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
  noProjects: boolean
}) {
  const classes = useStyles()
  const [formOpen, setFormOpen] = useState(noProjects)
  function handleProjectSubmit() {
    setFormOpen(false)
    refreshProjectList()
  }
  useEffect(() => {
    noProjects && setFormOpen(true)
  }, [noProjects])
  return (
    <div className={classes.projectControl} data-testid="project-control">
      <ProjectControlButtons
        onCreate={() => setFormOpen(!formOpen)}
        onRefresh={() => refreshProjectList()}
      />
      <ProjectCreateForm
        token={token}
        open={formOpen}
        onSuccess={handleProjectSubmit}
      />
    </div>
  )
}

function ProjectControlButtons({
  onCreate,
  onRefresh,
}: {
  onCreate: () => void
  onRefresh: () => void
}) {
  const classes = useStyles()
  const { promiseInProgress } = usePromiseTracker({ area: "get-projects" })
  return (
    <div
      className={classes.projectControlButtons}
      data-testid="project-control-buttons"
    >
      <ProjectCreateButton onClick={onCreate} />
      {promiseInProgress ? (
        <CircularProgress />
      ) : (
        <ProjectRefreshButton onClick={onRefresh} />
      )}
    </div>
  )
}

function ProjectList({
  token,
  projects,
  onRemove,
}: {
  token: string
  projects: Project[]
  onRemove?: () => void
}) {
  const classes = useStyles()
  return (
    <div className={classes.projectList} data-testid="project-list">
      {projects.length ? (
        <TableContainer>
          <Table>
            <TableHead>
              <StyledTableRow>
                <StyledTableCell align="center">Name</StyledTableCell>
                <StyledTableCell></StyledTableCell>
              </StyledTableRow>
            </TableHead>
            <TableBody>
              {projects.map((p) => (
                <ProjectEntry
                  key={p.user + p.name}
                  token={token}
                  project={p}
                  onRemove={onRemove}
                />
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      ) : (
        <NoProjects />
      )}
    </div>
  )
}

function ProjectEntry({
  token,
  project,
  onRemove,
}: {
  token: string
  project: Project
  onRemove?: () => void
}) {
  const { promiseInProgress } = usePromiseTracker({
    area: `remove-project-${project.name}`,
  })
  const [hideSelf, setHideSelf] = useState(false)
  const classes = useStyles()
  function handleClick() {
    trackPromise(
      deleteProject(token, project.name),
      `remove-project-${project.name}`
    ).then(() => {
      setHideSelf(true)
      onRemove?.()
    })
  }
  return (
    <StyledTableRow
      data-testid={`project-entry-${project.name}`}
      className={`${classes.projectEntry}${hideSelf ? " hidden" : ""}`}
    >
      <StyledTableCell align="center">{project.name}</StyledTableCell>
      <StyledTableCell>
        {promiseInProgress ? (
          <CircularProgress />
        ) : (
          <ProjectRemoveButton
            dataTestId={`project-remove-${project.name}`}
            onClick={handleClick}
          />
        )}
      </StyledTableCell>
    </StyledTableRow>
  )
}

function NoProjects() {
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
    <IconButton onClick={onClick} data-testid="project-create-button">
      <Add htmlColor={theme.palette.success.main} />
    </IconButton>
  )
}

function ProjectRefreshButton({ onClick }: { onClick?: () => void }) {
  return (
    <IconButton onClick={onClick} data-testid="project-refresh-button">
      <Refresh />
    </IconButton>
  )
}

function ProjectRemoveButton({
  onClick,
  dataTestId,
}: {
  onClick?: () => void
  dataTestId?: string
}) {
  const theme = useTheme()
  return (
    <IconButton data-testid={dataTestId} onClick={onClick}>
      <DeleteForever htmlColor={theme.palette.error.main} />
    </IconButton>
  )
}

function ProjectCreateForm({
  token,
  open,
  onSuccess,
}: {
  token: string
  open: boolean
  onSuccess?: () => void
}) {
  const classes = useStyles()
  const [name, setName] = useState("")
  const [errorMsg, setErrorMsg] = useState("")
  const { promiseInProgress } = usePromiseTracker({ area: "create-project" })

  function handleSubmit() {
    trackPromise(createProject(token, name), "create-project")
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
    <form
      className={`${classes.createForm}${open ? "" : " hidden"}`}
      data-testid="project-create-form"
    >
      <div>
        <TextField
          inputProps={{ "data-testid": "project-name-field" }}
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
            data-testid="create-project-submit"
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
