const initToc = () => {
  const intersections = {};

  document.querySelectorAll(".blog__section").forEach((el) => {
    intersections[el.id] = document.querySelector(`.blog__toc a[href="#${el.id}"]`);
  });

  Object.values(intersections).forEach((el) => {
    el.addEventListener("click", (event) => {
      event.preventDefault();
      const query = event.target.getAttribute("href");
      document.querySelector(query).scrollIntoView({
        behavior: "smooth",
      });
    });
  });



  //observer = new IntersectionObserver((entries) => {
  //  entries.forEach((entry) => {
  //    const id = entry.target.id;
  //    intersections[id].classList.toggle("active", entry.isIntersecting);
  //  });
  //},
  //  {
  //    rootMargin: "-30% 0px -60% 0px",
  //  });

  //document.querySelectorAll(".blog__section").forEach((el) => {
  //  observer.observe(el);
  //});
};

document.addEventListener("DOMContentLoaded", () => {
  initToc();
});
