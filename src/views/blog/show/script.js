const initToc = () => {
  const intersections = {};

  document.querySelectorAll(".blog__section").forEach((el) => {
    intersections[el.id] = document.querySelector(`.blog__toc a[href="#${el.id}"]`);
  });


  observer = new IntersectionObserver((entries) => {
    entries.forEach((entry) => {
      const id = entry.target.id;
      intersections[id].classList.toggle("active", entry.isIntersecting);
    });
  },
    {
      rootMargin: "-50% 0px",
    });

  document.querySelectorAll(".blog__section").forEach((el) => {
    observer.observe(el);
  });
};

document.addEventListener("DOMContentLoaded", () => {
  initToc();
});
