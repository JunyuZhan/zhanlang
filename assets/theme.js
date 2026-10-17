(function () {
  var KEY = "zhanlang-theme";
  var root = document.documentElement;

  // 按浏览者本地时间自动判断：6:00–17:59 浅色，其余深色
  function autoTheme() {
    var h = new Date().getHours();
    return h >= 6 && h < 18 ? "light" : "dark";
  }

  function current() {
    try {
      var saved = localStorage.getItem(KEY);
      if (saved === "light" || saved === "dark") return saved;
    } catch (e) {}
    return autoTheme();
  }

  function apply(t) {
    root.setAttribute("data-theme", t);
  }

  function toggle() {
    var next = root.getAttribute("data-theme") === "dark" ? "light" : "dark";
    try {
      localStorage.setItem(KEY, next);
    } catch (e) {}
    apply(next);
  }

  apply(current());
  window.__zhanlangToggleTheme = toggle;

  function bind() {
    var buttons = document.querySelectorAll("[data-theme-toggle]");
    for (var i = 0; i < buttons.length; i++) {
      buttons[i].addEventListener("click", toggle);
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", bind);
  } else {
    bind();
  }
})();
