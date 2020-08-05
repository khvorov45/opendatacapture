export function themeInit(): "dark" | "light" {
  let localtheme = localStorage.getItem("theme")
  if (!localtheme || !["dark", "light"].includes(localtheme)) {
    localStorage.setItem("theme", "dark")
    document.documentElement.setAttribute("theme", "dark")
    return "dark"
  } else if (localtheme === "dark") {
    document.documentElement.setAttribute("theme", "dark")
    return "dark"
  } else {
    document.documentElement.setAttribute("theme", "light")
    return "light"
  }
}
