const initTimestamps = () => {
  document.querySelectorAll('time').forEach((el) => {
    const timeStr = el.attributes.datetime;
    if (!timeStr || !timeStr.value) {
      return;
    }

    const date = new Date(timeStr.value);
    const formatted = new Intl.DateTimeFormat(undefined, {
      month: "short",
      day: "numeric",
      year: "numeric"
    }).format(date);

    el.innerText = formatted;
  })
};

document.addEventListener("DOMContentLoaded", () => {
  initTimestamps();
});
