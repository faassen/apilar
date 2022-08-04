export type World = {
  islands: Island[];
  observedIslandId: number;
  locations: Location[][];
};

export type Island = {
  width: number;
  height: number;
  totalFreeResources: number;
  totalBoundResources: number;
  totalMemoryResources: number;
  totalComputers: number;
  totalProcessors: number;
};

export type Location = {
  freeResources: number;
  computer: {
    memorySize: number;
    processors: number;
    boundResources: number;
  } | null;
};
