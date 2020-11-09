import React, { useMemo, useState } from "react"
import {
  Table as MaterialTable,
  TableHead,
  TableBody,
  createStyles,
  makeStyles,
  Theme,
  TextField,
} from "@material-ui/core"
import { useAsync, useAsyncCallback } from "react-async-hook"
import { createUser, getUsers, removeUser } from "../../lib/api/user"
import { EmailPassword, User } from "../../lib/api/auth"
import { useTable } from "react-table"
import {
  StyledTableCell,
  StyledTableRow,
  TableContainerCentered,
} from "../table"
import {
  ButtonArray,
  CreateButton,
  RefreshButton,
  CheckButton,
  DeleteButton,
} from "../button"

const useStyles = makeStyles((theme: Theme) =>
  createStyles({
    usersAdminWidget: {
      display: "flex",
      flexDirection: "column",
      justifyContent: "center",
    },
    userInput: {
      display: "flex",
      "&>*": {
        marginRight: 5,
      },
    },
  })
)

export default function Users({ token }: { token: string }) {
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

  // User deletion
  const handleDelete = useAsyncCallback(async (email) => {
    await removeUser(token, email)
    fetchUsers.execute(token)
  })

  // Input hiding
  const [hideInput, setHideInput] = useState(true)

  const classes = useStyles()
  return (
    <div className={classes.usersAdminWidget} data-testid="users-admin-widget">
      <ButtonArray errorMsg={`${fetchUsers.error?.message ?? ""}`}>
        <CreateButton onClick={() => setHideInput((old) => !old)} />
        <RefreshButton
          onClick={() => fetchUsers.execute(token)}
          inProgress={fetchUsers.loading}
          dataTestId="refresh-users-button"
        />
      </ButtonArray>
      <UserInput
        token={token}
        hidden={hideInput}
        onSubmit={() => fetchUsers.execute(token)}
      />
      <TableContainerCentered>
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
              <StyledTableCell></StyledTableCell>
            </StyledTableRow>
          </TableHead>
          <TableBody {...getTableBodyProps()}>
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
                  <StyledTableCell>
                    <DeleteButton
                      onClick={() => handleDelete.execute(row.original.email)}
                    />
                  </StyledTableCell>
                </StyledTableRow>
              )
            })}
          </TableBody>
        </MaterialTable>
      </TableContainerCentered>
    </div>
  )
}

function UserInput({
  token,
  hidden,
  onSubmit,
}: {
  token: string
  hidden: boolean
  onSubmit: () => void
}) {
  const [email, setEmail] = useState("")
  const [password, setPassword] = useState("")

  const handleSubmit = useAsyncCallback(async (cred: EmailPassword) => {
    await createUser(cred)
    onSubmit()
  })

  const classes = useStyles()
  return (
    <div className={`${classes.userInput} ${hidden ? "nodisplay" : ""}`}>
      <TextField
        inputProps={{ "data-testid": "user-email-field" }}
        label="Email"
        value={email}
        onChange={(e) => setEmail(e.target.value)}
      />
      <TextField
        inputProps={{ "data-testid": "user-password-field" }}
        label="Password"
        value={password}
        onChange={(e) => setPassword(e.target.value)}
        type="password"
      />
      <ButtonArray errorMsg={handleSubmit.error?.message}>
        <CheckButton
          onClick={() =>
            handleSubmit.execute({ email: email, password: password })
          }
          inProgress={handleSubmit.loading}
        />
      </ButtonArray>
    </div>
  )
}
