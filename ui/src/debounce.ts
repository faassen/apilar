// based on https://gist.github.com/fr-ser/ded7690b245223094cd876069456ed6c

import { createRoot } from "solid-js";

export function debounce<F extends Function>(func: F, wait: number): F {
  let timeoutID: number;

  if (!Number.isInteger(wait)) {
    throw new Error("Debounce has to be called with an integer wait");
  }

  // conversion through any necessary as it wont satisfy criteria otherwise
  return <any>function (this: any, ...args: any[]) {
    clearTimeout(timeoutID);
    const context = this;

    timeoutID = window.setTimeout(() => {
      createRoot(() => {
        func.apply(context, args);
      });
    }, wait);
  };
}
