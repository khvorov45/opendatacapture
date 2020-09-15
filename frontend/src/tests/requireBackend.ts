/** All of these tests assume that the backend is running as if it was
 * started with the --clean option
 */

/* istanbul ignore file */

import { Access, LoginFailure, tokenFetcher, tokenValidator } from "../lib/auth"
import {
  createProject,
  getUserProjects,
  deleteProject,
  TableMeta,
  createTable,
  getAllTableNames,
  getAllMeta,
  getTableMeta,
  removeTable,
  insertData,
  getTableData,
  removeAllTableData,
} from "../lib/project"

test("wrong token", async () => {
  expect.assertions(1)
  try {
    let res = await tokenValidator("123")
    console.log(`wrong token response: ${res}`)
  } catch (e) {
    expect(e.message).toBe('NoSuchToken("123")')
  }
})

test("wrong password", async () => {
  expect.assertions(1)
  try {
    let res = await tokenFetcher({
      email: "admin@example.com",
      password: "123",
    })
    console.log(`wrong password response: ${res}`)
  } catch (e) {
    expect(e.message).toBe(LoginFailure.WrongPassword)
  }
})

test("wrong email", async () => {
  expect.assertions(1)
  try {
    let res = await tokenFetcher({
      email: "user@example.com",
      password: "admin",
    })
    console.log(`wrong email response: ${res}`)
  } catch (e) {
    expect(e.message).toBe(LoginFailure.EmailNotFound)
  }
})

test("correct credentials", async () => {
  let token = await tokenFetcher({
    email: "admin@example.com",
    password: "admin",
  })
  let admin = await tokenValidator(token)
  expect(admin.access).toBe(Access.Admin)
  expect(admin.email).toBe("admin@example.com")
  expect(admin.id).toBe(1)
})

describe("bad token", () => {
  test("create project", async () => {
    expect.assertions(1)
    try {
      await createProject("123", "test")
    } catch (e) {
      expect(e.message).toBe('NoSuchToken("123")')
    }
  })
  test("remove project", async () => {
    expect.assertions(1)
    try {
      await deleteProject("123", "test")
    } catch (e) {
      expect(e.message).toBe('NoSuchToken("123")')
    }
  })
  test("get projects", async () => {
    expect.assertions(1)
    try {
      await getUserProjects("123")
    } catch (e) {
      expect(e.message).toBe('NoSuchToken("123")')
    }
  })
  test("create table", async () => {
    expect.assertions(1)
    try {
      await createTable("123", "test", { name: "test", cols: [] })
    } catch (e) {
      expect(e.message).toBe('NoSuchToken("123")')
    }
  })
  test("remove table", async () => {
    expect.assertions(1)
    try {
      await removeTable("123", "test", "test")
    } catch (e) {
      expect(e.message).toBe('NoSuchToken("123")')
    }
  })
  test("get meta", async () => {
    expect.assertions(1)
    try {
      await getAllMeta("123", "test")
    } catch (e) {
      expect(e.message).toBe('NoSuchToken("123")')
    }
  })
  test("get table meta", async () => {
    expect.assertions(1)
    try {
      await getTableMeta("123", "test", "test")
    } catch (e) {
      expect(e.message).toBe('NoSuchToken("123")')
    }
  })
  test("insert table data", async () => {
    expect.assertions(1)
    try {
      await insertData("123", "test", "test", [])
    } catch (e) {
      expect(e.message).toBe('NoSuchToken("123")')
    }
  })
  test("remove all table data", async () => {
    expect.assertions(1)
    try {
      await removeAllTableData("123", "test", "test")
    } catch (e) {
      expect(e.message).toBe('NoSuchToken("123")')
    }
  })
  test("get table data", async () => {
    expect.assertions(1)
    try {
      await getTableData("123", "test", "test")
    } catch (e) {
      expect(e.message).toBe('NoSuchToken("123")')
    }
  })
})

describe("need credentials", () => {
  let token: string

  beforeAll(async () => {
    token = await tokenFetcher({
      email: "admin@example.com",
      password: "admin",
    })
  })

  test("project manipulation", async () => {
    await createProject(token, "test")
    let projects = await getUserProjects(token)
    let projectIds = projects.map((p) => `${p.user}${p.name}`)
    expect(projectIds).toContain("1test")
    await deleteProject(token, "test")
    projects = await getUserProjects(token)
    projectIds = projects.map((p) => `${p.user}${p.name}`)
    expect(projectIds).not.toContain("1test")
  })

  test("create the same project twice", async () => {
    expect.assertions(1)
    await createProject(token, "test")
    try {
      await createProject(token, "test")
    } catch (e) {
      expect(e.message).toBe('ProjectAlreadyExists(1, "test")')
    }
    await deleteProject(token, "test")
  })

  test("delete nonexistent project", async () => {
    expect.assertions(1)
    try {
      await deleteProject(token, "nonexistent")
    } catch (e) {
      expect(e.message).toBe('NoSuchProject(1, "nonexistent")')
    }
  })

  describe("manipulate non-existent project", () => {
    test("create table", async () => {
      expect.assertions(1)
      try {
        await createTable(token, "nonexistent", { name: "sometable", cols: [] })
      } catch (e) {
        expect(e.message).toBe('NoSuchProject(1, "nonexistent")')
      }
    })
    test("delete table", async () => {
      expect.assertions(1)
      try {
        await removeTable(token, "nonexistent", "some-table")
      } catch (e) {
        expect(e.message).toBe('NoSuchProject(1, "nonexistent")')
      }
    })
    test("get table names", async () => {
      expect.assertions(1)
      try {
        await getAllTableNames(token, "nonexistent")
      } catch (e) {
        expect(e.message).toBe('NoSuchProject(1, "nonexistent")')
      }
    })
    test("get all meta", async () => {
      expect.assertions(1)
      try {
        await getAllMeta(token, "nonexistent")
      } catch (e) {
        expect(e.message).toBe('NoSuchProject(1, "nonexistent")')
      }
    })
    test("get table meta", async () => {
      expect.assertions(1)
      try {
        await getTableMeta(token, "nonexistent", "table")
      } catch (e) {
        expect(e.message).toBe('NoSuchProject(1, "nonexistent")')
      }
    })
    test("insert table data", async () => {
      expect.assertions(1)
      try {
        await insertData(token, "nonexistent", "table", [])
      } catch (e) {
        expect(e.message).toBe('NoSuchProject(1, "nonexistent")')
      }
    })
    test("remove all table data", async () => {
      expect.assertions(1)
      try {
        await removeAllTableData(token, "nonexistent", "table")
      } catch (e) {
        expect(e.message).toBe('NoSuchProject(1, "nonexistent")')
      }
    })
    test("get table data", async () => {
      expect.assertions(1)
      try {
        await getTableData(token, "nonexistent", "table")
      } catch (e) {
        expect(e.message).toBe('NoSuchProject(1, "nonexistent")')
      }
    })
  })

  const primaryTable: TableMeta = {
    name: "primary",
    cols: [
      {
        name: "id",
        postgres_type: "integer",
        not_null: true,
        unique: false, // UNIQUE constraint isn't added when PRIMARY KEY
        primary_key: true,
        foreign_key: null,
      },
      {
        name: "email",
        postgres_type: "text",
        not_null: true,
        unique: true,
        primary_key: false,
        foreign_key: null,
      },
    ],
  }

  const primaryData = [
    { id: 1, email: "email1@example.com" },
    { id: 2, email: "email2@example.com" },
  ]

  const secondaryTable: TableMeta = {
    name: "secondary",
    cols: [
      {
        name: "id",
        postgres_type: "integer",
        not_null: true,
        unique: false,
        primary_key: true,
        foreign_key: {
          table: "primary",
          column: "id",
        },
      },
      {
        name: "timepoint",
        postgres_type: "integer",
        not_null: true,
        unique: false,
        primary_key: true,
        foreign_key: null,
      },
    ],
  }

  describe("need a project", () => {
    beforeAll(async () => await createProject(token, "test"))
    afterAll(async () => await deleteProject(token, "test"))

    test("table manipulation", async () => {
      await createTable(token, "test", primaryTable)
      await createTable(token, "test", secondaryTable)
      let tableNames = await getAllTableNames(token, "test")
      expect(tableNames).toEqual([primaryTable.name, secondaryTable.name])
      let allMeta = await getAllMeta(token, "test")
      expect(allMeta).toEqual([primaryTable, secondaryTable])
      let primaryMeta = await getTableMeta(token, "test", "primary")
      expect(primaryMeta).toEqual(primaryTable)
      await removeTable(token, "test", "secondary")
      expect(await getAllTableNames(token, "test")).toEqual([primaryTable.name])
      expect(await getTableData(token, "test", primaryTable.name)).toEqual([])
      await insertData(token, "test", primaryTable.name, primaryData)
      expect(await getTableData(token, "test", primaryTable.name)).toEqual(
        primaryData
      )
      await removeAllTableData(token, "test", primaryTable.name)
      expect(await getTableData(token, "test", primaryTable.name)).toEqual([])
    })

    test("delete nonexistent table", async () => {
      expect.assertions(1)
      try {
        await removeTable(token, "test", "nonexistent")
      } catch (e) {
        expect(e.message).toBe('NoSuchTable("nonexistent")')
      }
    })
  })
})
