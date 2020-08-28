import React from "react"

export default function Home({ token }: { token: string | null }) {
  return (
    <div data-testid="homepage">This is the homepage with token {token}</div>
  )
}
