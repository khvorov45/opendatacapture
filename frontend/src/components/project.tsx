import {
  Drawer,
  List,
  ListItem,
  ListItemText,
  Toolbar,
} from "@material-ui/core"
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
  return (
    <div data-testid={`project-page-${name}`}>
      <SideBar />
    </div>
  )
}

function SideBar() {
  return (
    <Drawer variant="permanent">
      <Toolbar />
      <List>
        <ListItem button>
          <ListItemText primary="Tables" />
        </ListItem>
      </List>
    </Drawer>
  )
}
