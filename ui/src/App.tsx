import { Component, onCleanup, onMount } from "solid-js";

import * as pixi from "pixi.js";
import { Viewport } from "pixi-viewport";
import { Simple, SpatialHash } from "pixi-cull";

import { World, Location } from "./world";
import { renderWorld, updateWorld, BOX_SIZE, WorldShapes } from "./pcanvas";
import * as random from "./random";
// https://stackoverflow.com/questions/71743027/how-to-use-vite-hmr-api-with-pixi-js

const socket = new WebSocket("ws://localhost:3000/ws");

socket.addEventListener("open", (event) => {
  socket.send("hello server!");
});

// socket.addEventListener("message", (event) => {
//   const world: World = JSON.parse(event.data);
//   console.log("Got data", world);
// });

const App: Component = () => {
  // the problem is with big worlds - it creates so many pixi sprites,
  // pixi doesn't like it anymore, even with culling in place
  // but let's not worry about them for now
  // const world: World = {
  //   width: 100,
  //   height: 100,
  //   locations: [],
  // };

  // for (let y = 0; y < world.height; y++) {
  //   let row: Location[] = [];
  //   for (let x = 0; x < world.width; x++) {
  //     row.push({
  //       freeResources: 100,
  //       computer: null,
  //     });
  //   }
  //   world.locations.push(row);
  // }

  const handleUpdate = () => {
    socket.send("Hello world");
    // for (let y = 0; y < world.height; y++) {
    //   for (let x = 0; x < world.width; x++) {
    //     world.locations[y][x].computer = null;
    //   }
    // }
    // for (let i = 0; i < 1000; i++) {
    //   world.locations[random.range(0, world.height)][
    //     random.range(0, world.width)
    //   ].computer = {
    //     memorySize: 3,
    //     boundResources: 10,
    //     processors: 5,
    //   };
    // }
    // updateWorld(world, worldShapes);
  };

  let pixiContainer: HTMLDivElement | undefined;

  // application width & height needs to be the same as viewport
  // width and height otherwise we can't fully scroll to the right somehow
  let app = new pixi.Application({
    width: 900,
    height: 800,
    backgroundAlpha: 0,
    resolution: window.devicePixelRatio,
  });

  const viewport = new Viewport({
    screenWidth: 900,
    screenHeight: 800,
    worldWidth: 70 * BOX_SIZE,
    worldHeight: 40 * BOX_SIZE,
    // worldWidth: world.width * BOX_SIZE,
    // worldHeight: world.height * BOX_SIZE,
    interaction: app.renderer.plugins.interaction,
  });

  let worldShapes: WorldShapes | undefined;
  app.stage.addChild(viewport);

  viewport.clamp({
    left: true,
    top: true,
    right: true,
    bottom: true,
    underflow: "topleft",
  });

  viewport.bounce({});
  viewport.drag();

  // // const cull = new Simple();
  // cull.addList(viewport.children);
  // cull.cull(viewport.getVisibleBounds());

  // pixi.Ticker.shared.add(() => {
  //   if (viewport.dirty) {
  //     cull.cull(viewport.getVisibleBounds());
  //     viewport.dirty = false;
  //   }
  // });

  // let elapsed = 0.0;
  // let i = 0;
  // app.ticker.add((delta) => {
  //   elapsed += delta;

  //   i++;
  // });

  onMount(() => {
    pixiContainer?.appendChild(app.view);
    // cull.cull(viewport.getVisibleBounds());

    let worldShapes: WorldShapes | undefined;

    socket.addEventListener("message", (event) => {
      const world: World = JSON.parse(event.data);
      // console.log("Got data", world);

      if (worldShapes == null) {
        worldShapes = renderWorld(viewport, world);
      } else {
        updateWorld(world, worldShapes);
      }
    });
  });

  return (
    <>
      <button onClick={handleUpdate}>Update</button>
      <div ref={pixiContainer}></div>
    </>
  );
};

export default App;
