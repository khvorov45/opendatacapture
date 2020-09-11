import React from "react"
import { useParams } from "react-router-dom"

export default function ProjectPage() {
  let { name } = useParams<{ name: string }>()
  return <div data-testid={`project-page-${name}`}>Project page for {name}</div>
}
