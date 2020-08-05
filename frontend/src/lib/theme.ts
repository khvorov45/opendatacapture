export function theme_init() {
  let localtheme = localStorage.getItem("theme")
  if (!localtheme) {
    localStorage.setItem("theme", "dark")
    document.documentElement.setAttribute("theme", "dark")
  } else {
    document.documentElement.setAttribute("theme", localtheme)
  }
}
