function setTheme(theme) {
  if (theme === '') document.documentElement.removeAttribute('theme');
  else document.documentElement.setAttribute('theme', 'dark');
  localStorage.setItem('theme', theme);
}
setTheme(localStorage.getItem('theme') ?? '');
