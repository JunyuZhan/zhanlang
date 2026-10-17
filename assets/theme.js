(function () {
  var KEY = "zhanlang-theme";
  function current() {
    try {
      var saved = localStorage.getItem(KEY);
      if (saved === "light" || saved === "dark") return saved;
    } catch (e) {}
    return window.matchMedia &&
      window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
  }
  function apply(t) {
    document.documentElement.setAttribute("data-theme", t);
  }
  apply(current());

  function updateButtons() {
    var isDark = document.documentElement.getAttribute("data-theme") === "dark";
    var buttons = document.querySelectorAll("[data-theme-toggle]");
    for (var i = 0; i < buttons.length; i++) {
      buttons[i].textContent = isDark ? "☀️" : "🌙";
      buttons[i].setAttribute(
        "aria-label",
        isDark ? "Switch to light theme" : "Switch to dark theme"
      );
    }
  }

  window.__zhanlangToggleTheme = function () {
    var next =
      document.documentElement.getAttribute("data-theme") === "dark"
        ? "light"
        : "dark";
    try {
      localStorage.setItem(KEY, next);
    } catch (e) {}
    apply(next);
    updateButtons();
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", updateButtons);
  } else {
    updateButtons();
  }
})();
