import { Component, onCleanup, onMount } from "solid-js";

import * as pixi from "pixi.js";
import { Viewport } from "pixi-viewport";
import { Simple, SpatialHash } from "pixi-cull";

import { World, Location } from "./world";
import { renderWorld, updateWorld, BOX_SIZE } from "./pcanvas";
import * as random from "./random";
// https://stackoverflow.com/questions/71743027/how-to-use-vite-hmr-api-with-pixi-js

const App: Component = () => {
  // the problem is with big worlds - it creates so many pixi sprites,
  // pixi doesn't like it anymore, even with culling in place
  // we need to actually create show only a fraction of the world that's
  // visible
  const world: World = {
    width: 100,
    height: 100,
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
    for (let i = 0; i < 1000; i++) {
      world.locations[random.range(0, world.height)][
        random.range(0, world.width)
      ].computer = {
        memorySize: 3,
        boundResources: 10,
        processors: 5,
      };
    }
    updateWorld(world, worldShapes);
  };

  let pixiContainer: HTMLDivElement | undefined;

  let app = new pixi.Application({
    width: 800,
    height: 800,
  });

  const viewport = new Viewport({
    // screenWidth: 600,
    // screenHeight: 600,
    // screenWidth: app.view.offsetWidth,
    // screenHeight: app.view.offsetHeight,
    worldWidth: world.width * BOX_SIZE,
    worldHeight: world.height * BOX_SIZE,
    interaction: app.renderer.plugins.interaction,
  });
  viewport.on("moved", ({ viewport }) => {
    // console.log("moved", viewport.left, viewport.top);
  });
  let worldShapes = renderWorld(viewport, world);
  app.stage.addChild(viewport);

  viewport.clamp({
    left: true,
    top: true,
    right: viewport.worldWidth * 1.2,
    bottom: viewport.worldHeight * 1.2,
    underflow: "center",
  });
  viewport.drag();

  // const cull = new Simple();
  // cull.addList(viewport.children);
  // cull.cull(viewport.getVisibleBounds());

  // pixi.Ticker.shared.add(() => {
  //   if (viewport.dirty) {
  //     cull.cull(viewport.getVisibleBounds());
  //     viewport.dirty = false;
  //   }
  // });

  let elapsed = 0.0;
  let i = 0;
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
