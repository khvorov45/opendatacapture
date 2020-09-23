import { withStyles, Theme, createStyles } from "@material-ui/core/styles"
import { TableRow, TableCell } from "@material-ui/core"

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
