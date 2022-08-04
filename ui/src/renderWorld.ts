import * as pixi from "pixi.js";

import { Island, World, Location } from "./world";
import { Viewport } from "pixi-viewport";

const BOX_SIZE = 20;

type RenderData = {
  sprites: pixi.Sprite[][];
};

const DIM_GREY = 0x696969;
const GREY = 0x808080;
const LIGHT_GREY = 0xd3d3d3;
const BLACK = 0x000000;
const RED = 0xff0000;

export function renderWorld(
  viewport: Viewport,
  world: World,
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

export function updateWorld(world: World, renderData: RenderData) {
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

function getFill(location: Location): number {
  if (location.computer != null) {
    return RED;
  }
  const max = 1000;
  const resources = location.freeResources > max ? max : location.freeResources;
  return (resources / max) * 50 + (1 - resources / max) * 219;
}
