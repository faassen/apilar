export type World = {
  width: number;
  height: number;
  totalFreeResources: number;
  totalBoundResources: number;
  totalMemoryResources: number;
  totalComputers: number;
  totalProcessors: number;
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
