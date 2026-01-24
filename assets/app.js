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

document.addEventListener("DOMContentLoaded", () => {
  initTimestamps();
});
