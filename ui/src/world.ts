export type World = {
  width: number;
  height: number;
  locations: Location[][];
};

export type Location = {
  freeResources: number;
  computer: {
    memorySize: number;
    processors: number;
    boundResources: number;
  } | null;
};

export function getFill(location: Location): string {
  if (location.computer != null) {
    return "red";
  }
  if (location.freeResources > 5000) {
    return "darkgrey";
  } else if (location.freeResources > 2000) {
    return "grey";
  } else if (location.freeResources > 0) {
    return "lightgrey";
  } else {
    return "black";
  }
}
