import { withStyles, Theme, createStyles } from "@material-ui/core/styles"
import { TableRow, TableCell, TableContainer } from "@material-ui/core"

export const StyledTableCell = withStyles((theme: Theme) =>
  createStyles({
    root: {
      border: "0px",
    },
    head: {
      backgroundColor: "var(--palette-bg-alt)",
    },
  })
)(TableCell)

export const StyledTableRow = withStyles((theme: Theme) =>
  createStyles({
    root: {
      "&:nth-of-type(odd)": {
        backgroundColor: theme.palette.action.hover,
      },
    },
  })
)(TableRow)

export const TableContainerCentered = withStyles((theme: Theme) =>
  createStyles({
    root: {
      margin: "auto",
      width: "auto",
      "& table": {
        margin: "auto",
        width: "auto",
      },
    },
  })
)(TableContainer)
