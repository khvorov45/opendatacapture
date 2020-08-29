import React, { useEffect, useState } from "react"
import { getUserProjects, Project } from "../lib/project"

export default function Home({ token }: { token: string | null }) {
  return (
    <div data-testid="homepage">
      {token ? <ProjectList token={token} /> : "Loading"}
    </div>
  )
}

function ProjectList({ token }: { token: string }) {
  let [projects, setProjects] = useState<Project[]>([])
  useEffect(() => {
    getUserProjects(token).then((ps) => setProjects(ps))
  }, [token])
  return (
    <div data-testid="project-list">
      {projects.length
        ? projects.map((p) => <ProjectEntry project={p} />)
        : "No projects"}
    </div>
  )
}

function ProjectEntry({ project }: { project: Project }) {
  return <div>Project entry {JSON.stringify(project)}</div>
}
