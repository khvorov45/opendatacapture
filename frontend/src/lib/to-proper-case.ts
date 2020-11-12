export default function toProperCase(s: string) {
  let newString = s.replace(/[^\w]/g, " ")
  return newString.replace(
    /\w\S*/,
    (txt) => txt.charAt(0).toUpperCase() + txt.substr(1).toLowerCase()
  )
}
