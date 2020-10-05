import React from "react"

export default function DataPanel({
  token,
  projectName,
}: {
  token: string
  projectName: string
}) {
  return <div data-testid="data-panel">Data panel</div>
}
