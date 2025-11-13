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

const initSort = () => {
  const select = document.getElementById("sort");
  if (!select) {
    return;
  }

  select.addEventListener("change", (event) => {
    const url = new URL(window.location);
    url.searchParams.set('sort', event.target.value);
    window.location = url.search;
  });
}

document.addEventListener("DOMContentLoaded", () => {
  initTimestamps();
  initSort();
});
