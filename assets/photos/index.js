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

const recalculateHeights = () => {
  const containerWidth = container.getBoundingClientRect().width - 1;
  const gap = parseFloat(getComputedStyle(container).gap.replace('px', ''));

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
    photos.forEach(({ element }) => {
      const height = rowWidth(photos) / aspectRatio;
      element.style.height = `${height}px`;
    });
  });
};

recalculateHeights();

let windowWidth = window.innerWidth;
window.addEventListener("resize", () => {
  if (window.innerWidth === windowWidth) {
    return;
  }

  windowWidth = window.innerWidth;
  recalculateHeights();
});
