import Konva from "konva";

export type Location = {
  freeResources: number;
  computer: {
    memorySize: number;
    processors: number;
    boundResources: number;
  } | null;
  shape?: Konva.Rect;
};

export const BOX_SIZE = 20;

export type World = {
  width: number;
  height: number;
  locations: Location[][];
};

export type WorldShapes = {
  shapes: Konva.Shape[][];
};

function getFill(location: Location): string {
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

export function createWorldLayer(
  stage: Konva.Stage,
  world: World
): [Konva.Layer, WorldShapes] {
  const layer = new Konva.Layer();
  const shapes: WorldShapes = { shapes: [] };
  for (let iy = 0; iy < world.locations.length; iy++) {
    const row = world.locations[iy];
    const shapesRow: Konva.Shape[] = [];
    for (let ix = 0; ix < row.length; ix++) {
      const location = row[ix];
      const fill = getFill(location);
      const box = new Konva.Rect({
        id: `x${ix}-y${iy}`,
        x: ix * BOX_SIZE,
        y: iy * BOX_SIZE,
        width: BOX_SIZE,
        height: BOX_SIZE,
        fill: fill,
        stroke: "black",
      });
      shapesRow.push(box);
      layer.add(box);
    }
    shapes.shapes.push(shapesRow);
  }
  stage.add(layer);
  return [layer, shapes];
}

export function updateWorldLayer(
  layer: Konva.Layer,
  world: World,
  shapes: WorldShapes
) {
  console.log("start updating layer");
  let start = new Date().getTime();
  for (let iy = 0; iy < world.locations.length; iy++) {
    const row = world.locations[iy];
    for (let ix = 0; ix < row.length; ix++) {
      const location = row[ix];
      const box = shapes.shapes[iy][ix];
      const fill = getFill(location);
      if (box.fill() != fill) {
        box.fill(fill);
      }
    }
  }
  let beforeDraw = new Date().getTime();
  console.log("before draw:", beforeDraw - start);
  layer.draw();
  let afterDraw = new Date().getTime();
  console.log("draw itself", afterDraw - beforeDraw);
}

export function render(
  stage: Konva.Stage,
  world: World
): [Konva.Layer, WorldShapes] {
  const [layer, worldShapes] = createWorldLayer(stage, world);

  layer.draw();
  return [layer, worldShapes];
}
