import { Location } from "./world";
import { sequentialHexes } from "./colors";

const RED = 0xff0000;
const WHITE = 0xffffff;

export type FillScheme = (location: Location) => number;

function getDefaultFill(location: Location): number {
  if (location.computer != null) {
    return RED;
  }
  return getFreeResourcesFill(location);
}

function convert(colors: number[], value: number, max: number): number {
  const maxedResources = value >= max ? max - 1 : value;
  return colors[Math.floor(maxedResources / (max / colors.length))];
}

function getFreeResourcesFill(location: Location): number {
  return convert(sequentialHexes.Blues[9], location.freeResources, 500);
}

function getMemoryPerComputer(location: Location): number {
  if (location.computer == null) {
    return WHITE;
  }
  return convert(sequentialHexes.Reds[9], location.computer.memorySize, 1000);
}

function getProcessorsPerComputer(location: Location): number {
  if (location.computer == null) {
    return WHITE;
  }
  return convert(sequentialHexes.Purples[9], location.computer.processors, 10);
}

export const fillSchemes: { [key: string]: FillScheme } = {
  default: getDefaultFill,
  "free resources": getFreeResourcesFill,
  "memory per computer": getMemoryPerComputer,
  "processors per computer": getProcessorsPerComputer,
};
