/** All of these tests assume that the backend is running as if it was
 * started with the --clean option
 * The tests should clean up after themselves, so there shouldn't be a need
 * to restart the backend between runs (at least successful runs)
 */

/* istanbul ignore file */

import { createUser, getUsers, removeUser } from "../lib/api/user"
import {
  EmailPassword,
  LoginFailure,
  refreshToken,
  removeToken,
  tokenFetcher,
  tokenValidator,
} from "../lib/api/auth"
import { decodeUserTable } from "../lib/api/io-validation"
import {
  createProject,
  getUserProjects,
  deleteProject,
  createTable,
  getAllTableNames,
  getAllMeta,
  getTableMeta,
  removeTable,
  insertData,
  getTableData,
  removeAllTableData,
  TableMeta,
} from "../lib/api/project"
import { table1, table2, table1data, tableTitre, tableTitreData } from "./util"
import { defaultAdmin, user1Cred, defaultAdminCred, user1 } from "./data"

async function expectFailure(
  fn: (...args: any[]) => any,
  args: any[],
  expectedMessageStart: string,
  context?: string
) {
  try {
    let res = await fn(...args)
    throw Error(
      `function returned ${res} when supposed to fail ${context ?? ""}`
    )
  } catch (e) {
    expect(e.message).toStartWith(expectedMessageStart)
  }
}

test("wrong token", async () => {
  await expectFailure(tokenValidator, ["123"], 'NoSuchToken("123")')
})

test("wrong password", async () => {
  const wrongCred: EmailPassword = {
    email: defaultAdminCred.email,
    password: "123",
  }
  await expectFailure(tokenFetcher, [wrongCred], LoginFailure.WrongPassword)
})

test("wrong email", async () => {
  const wrongCred: EmailPassword = {
    email: "user@example.com",
    password: defaultAdminCred.password,
  }
  await expectFailure(tokenFetcher, [wrongCred], LoginFailure.EmailNotFound)
})

test("correct credentials", async () => {
  let token = await tokenFetcher(defaultAdminCred)
  let admin = await tokenValidator(token.token)
  expect(admin.access).toBe(defaultAdmin.access)
  expect(admin.email).toBe(defaultAdmin.email)
  expect(admin.id).toBe(defaultAdmin.id)
})

test("remove token", async () => {
  let token = await tokenFetcher(defaultAdminCred)
  await removeToken(token.token)
  expectFailure(tokenValidator, [token.token], "NoSuchToken")
})

describe("bad token", () => {
  const badToken = "123"

  async function expectNoSuchToken(
    fn: (...args: any[]) => any,
    args: any[],
    context?: string
  ) {
    await expectFailure(fn, args, `NoSuchToken("${badToken}")`, context)
  }

  test("create project", async () => {
    await expectNoSuchToken(createProject, [badToken, "test"])
  })

  test("remove project", async () => {
    await expectNoSuchToken(deleteProject, [badToken, "test"])
  })

  test("get projects", async () => {
    await expectNoSuchToken(getUserProjects, [badToken])
  })

  test("create table", async () => {
    const testTable: TableMeta = { name: "test", cols: [] }
    await expectNoSuchToken(createTable, [badToken, "test", testTable])
  })

  test("remove table", async () => {
    await expectNoSuchToken(removeTable, [badToken, "test", "test"])
  })

  test("get meta", async () => {
    await expectNoSuchToken(getAllMeta, [badToken, "test"])
  })

  test("get table meta", async () => {
    await expectNoSuchToken(getTableMeta, [badToken, "test", "test"])
  })

  test("insert table data", async () => {
    await expectNoSuchToken(insertData, [badToken, "test", "test", []])
  })

  test("remove all table data", async () => {
    await expectNoSuchToken(removeAllTableData, [badToken, "test", "test"])
  })

  test("get table data", async () => {
    await expectNoSuchToken(getTableData, [badToken, "test", "test"])
  })

  test("token refresh", async () => {
    await expectNoSuchToken(refreshToken, [badToken])
  })

  test("remove user", async () => {
    await expectNoSuchToken(removeUser, [badToken, "any@example.com"])
  })

  test("get users", async () => {
    await expectNoSuchToken(getUsers, [badToken])
  })
})

test("token refresh", async () => {
  let token = (await tokenFetcher(defaultAdminCred)).token
  const newTok = await refreshToken(token)
  expect(newTok).not.toEqual(token)
})

describe("need admin credentials", () => {
  let token: string

  beforeAll(async () => {
    token = (await tokenFetcher(defaultAdminCred)).token
  })

  test("get users", async () => {
    let users = await getUsers(token)
    expect(users).toEqual([defaultAdmin])
  })

  test("create/remove user", async () => {
    async function failTokenFetch(cred: EmailPassword, msg: string) {
      expectFailure(tokenFetcher, [cred], LoginFailure.EmailNotFound, msg)
    }
    // User shouldn't exist
    await failTokenFetch(user1Cred, "before creation")
    // Create them
    await createUser(user1Cred)
    // Token fetching should work
    const userToken = await tokenFetcher(user1Cred)
    // Check the user is who we expect them to be
    const user = await tokenValidator(userToken.token)
    expect(user.access).toBe(user1.access)
    expect(user.email).toBe(user1.email)
    // Creating them again should cause an error
    expectFailure(createUser, [user1Cred], "Request failed")
    // Remove user
    await removeUser(token, user1Cred.email)
    await failTokenFetch(user1Cred, "after creation")
  })
})

describe("need user credentials", () => {
  let token: string

  beforeAll(async () => {
    await createUser(user1Cred)
    token = (
      await tokenFetcher({
        email: user1Cred.email,
        password: user1Cred.password,
      })
    ).token
  })

  afterAll(async () => {
    const adminTok = (
      await tokenFetcher({
        email: defaultAdmin.email,
        password: "admin",
      })
    ).token
    await removeUser(adminTok, user1Cred.email)
  })

  describe("insufficient access", () => {
    async function expectInsufficientAccess(
      fn: (...args: any[]) => any,
      args: any[],
      context?: string
    ) {
      await expectFailure(fn, args, "InsufficientAccess", context)
    }

    test("get users", async () => {
      await expectInsufficientAccess(getUsers, [token])
    })

    test("remove user", async () => {
      await expectInsufficientAccess(removeUser, [token, "any@example.com"])
    })
  })

  test("project manipulation", async () => {
    const projectName = "test"
    await createProject(token, projectName)
    let projects = await getUserProjects(token)
    let projectIds = projects.map((p) => p.name)
    expect(projectIds).toContain(projectName)
    await deleteProject(token, projectName)
    projects = await getUserProjects(token)
    projectIds = projects.map((p) => p.name)
    expect(projectIds).not.toContain(projectName)
  })

  describe("manipulate non-existent project", () => {
    const prjName = "nonexistent"

    async function expectNoSuchProject(
      fn: (...args: any[]) => any,
      args: any[],
      context?: string
    ) {
      await expectFailure(fn, args, `NoSuchProject`, context)
    }

    test("delete nonexistent project", async () => {
      await expectNoSuchProject(deleteProject, [token, prjName])
    })

    test("create table", async () => {
      const testTable: TableMeta = { name: "sometable", cols: [] }
      await expectNoSuchProject(createTable, [token, prjName, testTable])
    })

    test("delete table", async () => {
      await expectNoSuchProject(removeTable, [token, prjName, "any"])
    })

    test("get table names", async () => {
      await expectNoSuchProject(getAllTableNames, [token, prjName])
    })

    test("get all meta", async () => {
      await expectNoSuchProject(getAllMeta, [token, prjName])
    })

    test("get table meta", async () => {
      await expectNoSuchProject(getTableMeta, [token, prjName, "any"])
    })

    test("insert table data", async () => {
      await expectNoSuchProject(insertData, [token, prjName, "any", []])
    })

    test("remove all table data", async () => {
      await expectNoSuchProject(removeAllTableData, [token, prjName, "any"])
    })

    test("get table data", async () => {
      await expectNoSuchProject(getTableData, [token, "nonexistent", "table"])
    })
  })

  describe("need a project", () => {
    const prjName = "test"

    beforeAll(async () => await createProject(token, prjName))
    afterAll(async () => await deleteProject(token, prjName))

    test("table manipulation", async () => {
      expect(await getAllTableNames(token, prjName)).toEqual([])
      await createTable(token, prjName, table1)
      await createTable(token, prjName, table2)
      expect(await getAllTableNames(token, prjName)).toEqual([
        table1.name,
        table2.name,
      ])
      let allMeta = await getAllMeta(token, prjName)
      expect(allMeta).toEqual([table1, table2])
      let primaryMeta = await getTableMeta(token, prjName, table1.name)
      expect(primaryMeta).toEqual(table1)
      await removeTable(token, prjName, table2.name)
      expect(await getAllTableNames(token, prjName)).toEqual([table1.name])
      await removeTable(token, prjName, table1.name)
      expect(await getAllTableNames(token, prjName)).toEqual([])
    })

    test("try to create again", async () => {
      await expectFailure(
        createProject,
        [token, prjName],
        "ProjectAlreadyExists"
      )
    })

    test("data push/pull from a table that has the same name as its column", async () => {
      await createTable(token, prjName, tableTitre)
      await insertData(token, prjName, tableTitre.name, tableTitreData)
      const dataObtained = await getTableData(token, prjName, tableTitre.name)
      expect(dataObtained).toEqual(tableTitreData)
    })

    describe("nonexistent table", () => {
      async function expectNoSuchTable(
        fn: (...args: any[]) => any,
        args: any[],
        context?: string
      ) {
        await expectFailure(fn, args, "NoSuchTable", context)
      }

      test("delete", async () => {
        await expectNoSuchTable(removeTable, [token, prjName, "nonexistent"])
      })

      test("get meta", async () => {
        await expectNoSuchTable(getTableMeta, [token, prjName, "nonexistent"])
      })

      test("insert", async () => {
        await expectNoSuchTable(insertData, [token, prjName, "nonexistent", []])
      })

      test("get data", async () => {
        await expectNoSuchTable(getTableData, [token, prjName, "nonexistent"])
      })

      test("remove all data", async () => {
        await expectNoSuchTable(removeAllTableData, [
          token,
          prjName,
          "nonexistent",
        ])
      })
    })

    describe("need a table", () => {
      beforeAll(async () => await createTable(token, prjName, table1))
      afterAll(async () => await removeTable(token, prjName, table1.name))

      test("data manipulation", async () => {
        expect(await getTableData(token, prjName, table1.name)).toEqual([])
        await insertData(token, prjName, table1.name, table1data)
        // The way the backend serializes dates is not the same as the
        // frontend does it
        expect(
          decodeUserTable(
            table1,
            await getTableData(token, prjName, table1.name)
          )
        ).toEqual(decodeUserTable(table1, table1data))
        await removeAllTableData(token, prjName, table1.name)
        expect(await getTableData(token, prjName, table1.name)).toEqual([])
      })

      test("insert data with the wrong columns", async () => {
        await expectFailure(
          insertData,
          [token, prjName, table1.name, [{ wrong: 1 }]],
          'NoSuchColumns(["wrong"])'
        )
      })

      test("try to create again", async () => {
        await expectFailure(
          createTable,
          [token, prjName, table1],
          `TableAlreadyExists("${table1.name}")`
        )
      })
    })
  })
})
