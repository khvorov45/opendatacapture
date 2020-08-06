import axios from "axios"

export interface EmailPassword {
  email: string
  password: string
}

export interface IdToken {
  id: number
  token: string
}

export async function sendEmailPassword(cred: EmailPassword) {
  const myHeaders = new Headers()
  myHeaders.append("Content-Type", "application/json")
  myHeaders.append("Accept", "application/json")
  const res = await axios.post(
    "http://localhost:4321/authenticate/email-password",
    cred
  )
  return res.data
}
