import { Component, onCleanup, onMount } from "solid-js";

import * as pixi from "pixi.js";
import { Viewport } from "pixi-viewport";
import { Simple, SpatialHash } from "pixi-cull";

import { World, Location } from "./world";
import { renderWorld, updateWorld, BOX_SIZE } from "./pcanvas";
// https://stackoverflow.com/questions/71743027/how-to-use-vite-hmr-api-with-pixi-js

const App: Component = () => {
  const world: World = {
    width: 40,
    height: 40,
    locations: [],
  };

  for (let y = 0; y < world.height; y++) {
    let row: Location[] = [];
    for (let x = 0; x < world.width; x++) {
      row.push({
        freeResources: 100,
        computer: null,
      });
    }
    world.locations.push(row);
  }

  const handleUpdate = () => {
    console.log("updating world");
    for (let iy = 10; iy < 20; iy++) {
      for (let ix = 10; ix < 20; ix++) {
        world.locations[iy][ix].computer = {
          memorySize: 3,
          boundResources: 10,
          processors: 5,
        };
      }
    }
    updateWorld(world, worldShapes);
  };

  let pixiContainer: HTMLDivElement | undefined;

  let app = new pixi.Application();

  const viewport = new Viewport({
    // screenWidth: app.view.offsetWidth,
    // screenHeight: app.view.offsetHeight,
    worldWidth: world.width * BOX_SIZE,
    worldHeight: world.height * BOX_SIZE,
    interaction: app.renderer.plugins.interaction,
  });
  let worldShapes = renderWorld(viewport, world);
  app.stage.addChild(viewport);

  viewport.drag();

  const cull = new Simple();
  cull.addList(viewport.children);
  cull.cull(viewport.getVisibleBounds());

  let elapsed = 0.0;
  let i = 0;

  pixi.Ticker.shared.add(() => {
    if (viewport.dirty) {
      cull.cull(viewport.getVisibleBounds());
      viewport.dirty = false;
    }
  });
  app.ticker.add((delta) => {
    elapsed += delta;

    i++;
  });

  onMount(() => {
    pixiContainer?.appendChild(app.view);
    // cull.cull(viewport.getVisibleBounds());
  });

  return (
    <>
      <button onClick={handleUpdate}>Update</button>
      <div ref={pixiContainer}></div>
    </>
  );
};

export default App;
