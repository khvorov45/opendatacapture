import React, { useMemo, useState } from "react"
import {
  CircularProgress,
  TableContainer,
  Table as MaterialTable,
  TableHead,
  TableBody,
  createStyles,
  makeStyles,
  Theme,
} from "@material-ui/core"
import { Redirect, Route, useLocation, useRouteMatch } from "react-router-dom"
import { SimpleNav } from "./nav"
import { useAsync } from "react-async-hook"
import { getUsers } from "../lib/api/admin"
import { User } from "../lib/api/auth"
import { useTable } from "react-table"
import {
  StyledTableCell,
  StyledTableRow,
  TableContainerCentered,
} from "./table"
import {
  ButtonArray,
  CreateButton,
  RefreshButton,
  DeleteButton,
  CheckButton,
} from "./button"

const useStyles = makeStyles((theme: Theme) =>
  createStyles({
    tableContainer: {
      width: "auto",
      "& table": {
        margin: "auto",
        width: "auto",
        "& td, & th": {
          textAlign: "center",
        },
      },
    },
  })
)

export default function AdminDashboard({ token }: { token: string | null }) {
  const { pathname } = useLocation()
  return (
    <div data-testid="admin-dashboard">
      <SimpleNav
        links={["users", "all-projects"]}
        dataTestId="project-page-links"
        active={(l) => pathname.startsWith(`/admin/${l}`)}
      />
      {token ? <Main token={token} /> : <CircularProgress />}
    </div>
  )
}

function Main({ token }: { token: string }) {
  const { url } = useRouteMatch()
  return (
    <div>
      <Route exact path={url}>
        <Redirect to={`${url}/users`} />
      </Route>
      <Route path={`${url}/users`}>
        <Users token={token} />
      </Route>
      <Route path={`${url}/all-projects`}>
        <Projects token={token} />
      </Route>
    </div>
  )
}

function Users({ token }: { token: string }) {
  const fetchUsers = useAsync(getUsers, [token], {
    setLoading: (state) => ({ ...state, loading: true }),
  })
  const users: User[] = useMemo(() => fetchUsers.result ?? [], [fetchUsers])
  const columns = useMemo(() => {
    return [
      {
        Header: "ID",
        accessor: (u: User) => u.id,
      },
      {
        Header: "Email",
        accessor: (u: User) => u.email,
      },
      {
        Header: "Access group",
        accessor: (u: User) => u.access.toString(),
      },
    ]
  }, [])
  const {
    headers,
    rows,
    getTableProps,
    getTableBodyProps,
    prepareRow,
  } = useTable<User>({
    columns: columns,
    data: users,
  })

  // Input hiding
  const [hideInput, setHideInput] = useState(true)

  return (
    <TableContainerCentered data-testid="users-admin-widget">
      <MaterialTable {...getTableProps()}>
        <TableHead>
          <StyledTableRow data-testid="header-row">
            {/*Actual headers*/}
            {headers.map((header) => (
              <StyledTableCell {...header.getHeaderProps()}>
                {header.render("Header")}
              </StyledTableCell>
            ))}
            {/*Control buttons*/}
            <StyledTableCell>
              <ButtonArray errorMsg={`${fetchUsers.error?.message ?? ""}`}>
                <CreateButton onClick={() => setHideInput((old) => !old)} />
                <RefreshButton
                  onClick={() => fetchUsers.execute(token)}
                  inProgress={fetchUsers.loading}
                  dataTestId="refresh-users-button"
                />
              </ButtonArray>
            </StyledTableCell>
          </StyledTableRow>
        </TableHead>
        <TableBody {...getTableBodyProps()}>
          <UserInputRow token={token} hidden={hideInput} />
          {rows.map((row) => {
            prepareRow(row)
            return (
              <StyledTableRow {...row.getRowProps()} data-testid="user-row">
                {row.cells.map((cell) => (
                  <StyledTableCell {...cell.getCellProps()}>
                    {cell.render("Cell")}
                  </StyledTableCell>
                ))}
                {/*Line up with control*/}
                <StyledTableCell />
              </StyledTableRow>
            )
          })}
        </TableBody>
      </MaterialTable>
    </TableContainerCentered>
  )
}

function UserInputRow({ token, hidden }: { token: string; hidden: boolean }) {
  return (
    <StyledTableRow className={hidden ? "nodisplay" : ""}>
      {/*ID*/}
      <StyledTableCell />
      <StyledTableCell>Email</StyledTableCell>
      <StyledTableCell>AccessGroup</StyledTableCell>
      <StyledTableCell>
        <CheckButton />
      </StyledTableCell>
    </StyledTableRow>
  )
}

function Projects({ token }: { token: string }) {
  return <div data-testid="projects-admin-widget">Projects</div>
}
