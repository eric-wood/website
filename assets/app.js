const colorschemes = {
  red: {
    foreground: "firebrick",
    background: "whitesmoke",
  },
  green: {
    foreground: "lightgreen",
    background: "darkslategray",
  },
  pink: {
    foreground: "pink",
    background: "rebeccapurple",
  },
  blue: {
    foreground: "midnightblue",
    background: "aliceblue",
  },
  brown: {
    foreground: "maroon",
    background: "oldlace",
  }
}

const longFormatter = new Intl.DateTimeFormat(undefined, {
  month: "short",
  day: "numeric",
  year: "numeric",
});

const shortFormatter = new Intl.DateTimeFormat(undefined, {
  month: "2-digit",
  day: "2-digit",
  year: "2-digit",
});

const timeFormatter = new Intl.DateTimeFormat(undefined, {
  dateStyle: "medium",
  timeStyle: "medium",
});

const initTimestamps = () => {
  document.querySelectorAll('time').forEach((el) => {
    let formatter = longFormatter
    const isShort = el.dataset.short === "true";
    const hasTime = el.dataset.time === "true";
    if (isShort) {
      formatter = shortFormatter;
    } else if (hasTime) {
      formatter = timeFormatter;
    }

    const timeStr = el.attributes.datetime;
    if (!timeStr || !timeStr.value) {
      return;
    }

    const date = new Date(timeStr.value);
    const formatted = formatter.format(date);

    el.innerText = formatted;
  })
};

const initTheme = () => {
  const themeCookie = document.cookie
    .split("; ")
    .find((row) => row.startsWith("theme="));

  let theme = "black";
  if (themeCookie) {
    theme = themeCookie.split("=")[1];
  }

  console.log(theme)
  const swatch = document.querySelector(`.theme-swatch[data-foreground="${theme}"]`);
  if (swatch) {
    setTheme(swatch);
  }
}

document.addEventListener("DOMContentLoaded", () => {
  initTimestamps();
  initTheme();
});

window.setTheme = (el) => {
  document.querySelector(".theme-swatch.selected")?.classList.remove("selected");
  el.classList.add("selected");

  const foreground = el.dataset.foreground;
  const background = el.dataset.background;
  document.body.style.setProperty("--foreground", foreground);
  document.body.style.setProperty("--background", background);

  document.cookie = `theme=${foreground}`;
}
