import axios from "axios"

export interface EmailPassword {
  email: string
  password: string
}

export interface IdToken {
  id: number
  token: string
}

export interface Token {
  user: number
  token: string
  created: Date
}

export async function sendEmailPassword(cred: EmailPassword): Promise<Token> {
  const myHeaders = new Headers()
  myHeaders.append("Content-Type", "application/json")
  myHeaders.append("Accept", "application/json")
  const res = await axios.post(
    "http://localhost:4321/authenticate/email-password",
    cred
  )
  // Should be an object with Ok field if successful
  if (typeof res.data === "string") {
    throw Error(res.data)
  }
  if (
    !res.data.Ok ||
    !res.data.Ok.user ||
    !res.data.Ok.token ||
    !res.data.Ok.created
  ) {
    throw Error("unexpected response data")
  }
  return {
    user: res.data.Ok.user,
    token: res.data.Ok.token,
    created: new Date(res.data.Ok.created),
  }
}
