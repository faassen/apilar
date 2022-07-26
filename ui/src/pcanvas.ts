import * as pixi from "pixi.js";

import { World, Location } from "./world";
import { Viewport } from "pixi-viewport";
import { Sprite } from "pixi.js";

export const BOX_SIZE = 20;

export type WorldShapes = {
  shapes: pixi.Sprite[][];
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
  sprite: pixi.Sprite,
  x: number,
  y: number,
  size: number,
  fill: number
) {
  sprite.tint = fill;
  sprite.position.set(x * size, y * size);
  sprite.width = size;
  sprite.height = size;
}

export function renderWorld(viewport: Viewport, world: World): WorldShapes {
  const shapes: WorldShapes = { shapes: [] };
  for (let iy = 0; iy < world.locations.length; iy++) {
    const row = world.locations[iy];
    const shapesRow: pixi.Sprite[] = [];
    for (let ix = 0; ix < row.length; ix++) {
      const location = row[ix];
      const fill = getFill(location);
      const sprite = new pixi.Sprite(pixi.Texture.WHITE);
      drawBox(sprite, ix, iy, BOX_SIZE, fill);
      shapesRow.push(sprite);
      viewport.addChild(sprite);
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
      graphics.tint = fill;
    }
  }
}
