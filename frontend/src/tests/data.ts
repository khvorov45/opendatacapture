/* istanbul ignore file */

import { Access, EmailPassword, User } from "../lib/api/auth"

export const defaultAdmin: User = {
  id: 1,
  email: "admin@example.com",
  access: Access.Admin,
}

export const user1: User = {
  id: 2,
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
