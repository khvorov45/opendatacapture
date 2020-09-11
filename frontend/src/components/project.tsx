import React, { useEffect } from "react"
import { useParams } from "react-router-dom"

export default function ProjectPage({
  onVisit,
}: {
  onVisit: (projectName: string) => void
}) {
  let { name } = useParams<{ name: string }>()
  useEffect(() => {
    onVisit?.(name)
  }, [name, onVisit])
  return <div data-testid={`project-page-${name}`}>Project page for {name}</div>
}
