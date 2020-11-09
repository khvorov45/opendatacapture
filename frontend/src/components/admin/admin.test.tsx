/* istanbul ignore file */
import axios from "axios"
import { fireEvent, wait } from "@testing-library/react"
import { renderAdminPage } from "../../tests/util"
import { constructGet } from "../../tests/api"

jest.mock("axios")
const mockedAxios = axios as jest.Mocked<typeof axios>

mockedAxios.get.mockImplementation(constructGet())

test("navigation", async () => {
  const admin = renderAdminPage()
  await wait(() =>
    expect(admin.getByTestId("users-admin-widget")).toBeInTheDocument()
  )
  fireEvent.click(admin.getByText("All projects"))
  await wait(() => {
    expect(admin.getByTestId("projects-admin-widget")).toBeInTheDocument()
  })
})
