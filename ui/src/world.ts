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
