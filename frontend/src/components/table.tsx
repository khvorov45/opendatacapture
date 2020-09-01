import { withStyles, Theme, createStyles } from "@material-ui/core/styles"
import { TableRow, TableCell } from "@material-ui/core"

export const StyledTableCell = withStyles((theme: Theme) =>
  createStyles({
    root: {
      border: "0px",
    },
    head: {
      backgroundColor:
        /* istanbul ignore next */
        theme.palette.type === "dark"
          ? theme.palette.grey[900]
          : theme.palette.grey[300],
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
