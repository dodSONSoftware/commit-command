import "@testing-library/jest-dom/vitest";

// jsdom supplies the element but does not implement the native modal methods.
HTMLDialogElement.prototype.showModal ??= function () {
  this.open = true;
};
HTMLDialogElement.prototype.close ??= function () {
  this.open = false;
};
