import { Location } from "./world";
import { sequentialHexes } from "./colors";

const RED = 0xff0000;

export function getDefaultFill(location: Location): number {
  if (location.computer != null) {
    return RED;
  }
  return getFreeResourcesFill(location);
}

export function getFreeResourcesFill(location: Location): number {
  const max = 500;
  const resources =
    location.freeResources > max ? max - 1 : location.freeResources;
  return sequentialHexes.Blues[9][Math.floor(resources / (max / 9))];
}

export type FillScheme = (location: Location) => number;

export const fillSchemes: { [key: string]: FillScheme } = {
  default: getDefaultFill,
  "free-resources": getFreeResourcesFill,
};
