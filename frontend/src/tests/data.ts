/* istanbul ignore file */

import { Access, EmailPassword, Token, User } from "../lib/api/auth"
import { TableMeta, TableData } from "../lib/api/project"

export const defaultAdmin: User = {
  id: 1, // Should be the same on fetch because it's the first user created
  email: "admin@example.com",
  access: Access.Admin,
}

export const user1: User = {
  id: 2, // May be different on fetch since we'll be creating/dropping users
  email: "user1@xample.com",
  access: Access.User,
}

export const defaultAdminCred: EmailPassword = {
  email: defaultAdmin.email,
  password: "admin",
}

export const user1Cred: EmailPassword = {
  email: user1.email,
  password: "user",
}

export const adminToken: Token = {
  user: defaultAdmin.id,
  token: "123",
  created: new Date(),
}

/* NOTE on postgres and primary keys:

When a column is marked as PRIMARY_KEY, the UNIQUE constraint is always absent
and the NOT_NULL "constraint" is always present regardless of what
SQL you write.

So all of the following will produce the same table:

CREATE TABLE some_table (id INT PRIMARY KEY)
CREATE TABLE some_table (id INT PRIMARY KEY NOT_NULL)
CREATE TABLE some_table (id INT PRIMARY KEY UNIQUE)
CREATE TABLE some_table (id INT PRIMARY KEY NOT_NULL UNIQUE)
*/

export const table1: TableMeta = {
  name: "newtable",
  cols: [
    {
      name: "id",
      postgres_type: "integer",
      primary_key: true,
      not_null: true,
      unique: false,
      foreign_key: null,
    },
    {
      name: "email",
      postgres_type: "text",
      primary_key: false,
      not_null: true,
      unique: true,
      foreign_key: null,
    },
    {
      name: "height",
      postgres_type: "real",
      primary_key: false,
      not_null: false,
      unique: false,
      foreign_key: null,
    },
    {
      name: "weight",
      postgres_type: "real",
      primary_key: false,
      not_null: false,
      unique: false,
      foreign_key: null,
    },
    {
      name: "male",
      postgres_type: "boolean",
      primary_key: false,
      not_null: false,
      unique: false,
      foreign_key: null,
    },
    {
      name: "dob",
      postgres_type: "timestamp with time zone",
      primary_key: false,
      not_null: false,
      unique: false,
      foreign_key: null,
    },
  ],
}

/* NOTE on dates:

Received dates are only decoded before presentation
Sent dates are decoded on input (JSON encoding sends them as strings though)
*/

export const table1data: TableData = [
  {
    id: 1,
    email: "e1@example.com",
    height: 170.5,
    weight: 60,
    male: false,
    dob: "2020-01-01T00:00:00.000Z",
  },
  {
    id: 2,
    email: "e2@example.com",
    height: 180,
    weight: 70.0,
    male: true,
    dob: "2020-01-01T00:00:00.000Z",
  },
]

// Compound primary key
export const table2: TableMeta = {
  name: "newtable2",
  cols: [
    {
      name: "id",
      postgres_type: "integer",
      primary_key: true,
      not_null: true,
      unique: false,
      foreign_key: { table: table1.name, column: table1.cols[0].name },
    },
    {
      name: "timepoint",
      postgres_type: "text",
      primary_key: true,
      not_null: true,
      unique: false,
      foreign_key: null,
    },
  ],
}

export const table2data = [
  { id: 1, timepoint: 1 },
  { id: 1, timepoint: 2 },
]

// No primary key
export const table3: TableMeta = {
  name: "newtable3",
  cols: [
    {
      name: "id",
      postgres_type: "integer",
      primary_key: false,
      not_null: false,
      unique: true,
      foreign_key: null,
    },
  ],
}

export const table3data: TableData = [{ id: 1 }]

// Has the same name as one of its columns
export const tableTitre: TableMeta = {
  name: "titre",
  cols: [
    {
      name: "titre",
      postgres_type: "integer",
      primary_key: false,
      not_null: false,
      unique: false,
      foreign_key: null,
    },
  ],
}

export const tableTitreData: TableData = [{ titre: 20 }, { titre: 40 }]

export const allTables: { meta: TableMeta; data: TableData }[] = [
  { meta: table1, data: table1data },
  { meta: table2, data: table2data },
  { meta: table3, data: table3data },
  { meta: tableTitre, data: tableTitreData },
]
