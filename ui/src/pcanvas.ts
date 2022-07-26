import * as pixi from "pixi.js";

import { World, Location } from "./world";
import { Viewport } from "pixi-viewport";

export const BOX_SIZE = 20;

export type WorldShapes = {
  shapes: pixi.Graphics[][];
};

const DIM_GREY = 0x696969;
const GREY = 0x808080;
const LIGHT_GREY = 0xd3d3d3;
const BLACK = 0x000000;
const RED = 0xff0000;

export function getFill(location: Location): number {
  if (location.computer != null) {
    return RED;
  }
  if (location.freeResources > 5000) {
    return DIM_GREY;
  } else if (location.freeResources > 2000) {
    return GREY;
  } else if (location.freeResources > 0) {
    return LIGHT_GREY;
  } else {
    return BLACK;
  }
}

function drawBox(
  graphics: pixi.Graphics,
  x: number,
  y: number,
  size: number,
  fill: number
) {
  // black outline box
  graphics.beginFill(BLACK);
  graphics.drawRect(x * size, y * size, size, size);
  // inner box
  graphics.beginFill(fill);
  graphics.drawRect(x * size + 1, y * size + 1, size - 1, size - 1);
}

export function renderWorld(viewport: Viewport, world: World): WorldShapes {
  const shapes: WorldShapes = { shapes: [] };
  for (let iy = 0; iy < world.locations.length; iy++) {
    const row = world.locations[iy];
    const shapesRow: pixi.Graphics[] = [];
    for (let ix = 0; ix < row.length; ix++) {
      const location = row[ix];
      const fill = getFill(location);
      const graphics = new pixi.Graphics();
      drawBox(graphics, ix, iy, BOX_SIZE, fill);
      shapesRow.push(graphics);
      viewport.addChild(graphics);
    }
    shapes.shapes.push(shapesRow);
  }
  return shapes;
}

export function updateWorld(world: World, shapes: WorldShapes) {
  for (let iy = 0; iy < world.locations.length; iy++) {
    const row = world.locations[iy];
    for (let ix = 0; ix < row.length; ix++) {
      const location = row[ix];
      const graphics = shapes.shapes[iy][ix];
      const fill = getFill(location);
      graphics.clear();
      drawBox(graphics, ix, iy, BOX_SIZE, fill);
    }
  }
}
