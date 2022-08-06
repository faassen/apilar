import * as pixi from "pixi.js";

import { World, Location } from "./world";
import { Viewport } from "pixi-viewport";
import { sequential_hexes } from "./colors";

const BOX_SIZE = 20;

export type RenderData = {
  sprites: pixi.Sprite[][];
};

const RED = 0xff0000;

export function renderWorld(
  viewport: Viewport,
  world: World,
  getFill: (location: Location) => number,
  onClick: (options: { location: Location; x: number; y: number }) => void
): RenderData {
  const renderData: RenderData = { sprites: [] };
  for (let iy = 0; iy < world.locations.length; iy++) {
    const row = world.locations[iy];
    const renderRow: pixi.Sprite[] = [];
    for (let ix = 0; ix < row.length; ix++) {
      const location = row[ix];
      const fill = getFill(location);
      const sprite = new pixi.Sprite(pixi.Texture.WHITE);
      drawBox(sprite, ix, iy, BOX_SIZE, fill);
      sprite.interactive = true;
      sprite.on("pointerdown", () => {
        onClick({ location, x: ix, y: iy });
      });
      renderRow.push(sprite);
      viewport.addChild(sprite);
    }
    renderData.sprites.push(renderRow);
  }
  return renderData;
}

export function updateWorld(
  world: World,
  renderData: RenderData,
  getFill: (location: Location) => number
) {
  for (let iy = 0; iy < world.locations.length; iy++) {
    const row = world.locations[iy];
    for (let ix = 0; ix < row.length; ix++) {
      const location = row[ix];
      const graphics = renderData.sprites[iy][ix];
      const fill = getFill(location);
      graphics.tint = fill;
    }
  }
}

export function getWorldDimensions(world: World): [number, number] {
  const island = world.islands[world.observedIslandId];
  // XXX there is a problem if we have islands of different dimensions,
  // because we only calculate this once and we only do render once, we need to
  // tweak it

  return [island.width * BOX_SIZE, island.height * BOX_SIZE];
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
  return sequential_hexes.Blues[9][Math.floor(resources / (max / 9))];
}

export type FillScheme = (location: Location) => number;

export const fillSchemes: { [key: string]: FillScheme } = {
  default: getDefaultFill,
  "free-resources": getFreeResourcesFill,
};
