const photos = Array.from(document.querySelectorAll(".photos > *")).map(
  (photo) => {
    const width = parseFloat(photo.dataset.width);
    const height = parseFloat(photo.dataset.height);

    return {
      id: photo.id,
      aspectRatio: width / height,
      element: photo,
    };
  }
);

const MAX_HEIGHT = parseFloat(getComputedStyle(photos[0].element).maxHeight.replace('px', ''))
const container = document.querySelector(".photos");
const gap = parseFloat(getComputedStyle(container).gap.replace('px', ''));

const recalculateHeights = () => {
  const containerWidth = container.getBoundingClientRect().width - 1;
  const newRow = () => ({
    photos: [],
    aspectRatio: 0,
  });

  const rowWidth = (photos) => {
    const totalGap = (photos.length - 1) * gap;
    return containerWidth - totalGap;
  }

  const grid = [newRow()];
  let currentRow = 0;
  photos.forEach((photo) => {
    const row = grid[currentRow];
    const aspectRatio = row.aspectRatio + photo.aspectRatio;
    row.aspectRatio = aspectRatio;
    row.photos.push(photo);

    const height = rowWidth(row.photos) / aspectRatio;
    if (height <= MAX_HEIGHT) {
      currentRow += 1;
      grid[currentRow] = newRow();
      return;
    }
  })

  grid.forEach(({ aspectRatio, photos }) => {
    photos.forEach((photo) => {
      const { element, aspectRatio: photoAspectRatio } = photo;
      photo.height = rowWidth(photos) / aspectRatio;
      photo.width = photoAspectRatio * photo.height;
    });
  });

  photos.forEach(({ element, width, height }) => {
    element.style.height = `${height}px`;
    element.style.maxWidth = `${width}px`;
    element.style.display = 'initial';
  });
};

const resizeObserver = new ResizeObserver((entries) => {
  for (const entry of entries) {
    recalculateHeights();
  }
});

resizeObserver.observe(document.body);

const pageNumberSelect = document.getElementById("page_number")
pageNumberSelect.addEventListener("change", (event) => {
  const page = event.currentTarget.value;
  const url = new URL(window.location);
  url.searchParams.set("page", page);
  window.location = url.toString();
});

document.querySelectorAll(".mobile-nav-toggle").forEach((el) => {
  el.addEventListener("click", () => {
    document.querySelector(".photos__nav").classList.toggle("open");
  });
});

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
  initSort();
});
