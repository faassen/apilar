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

const App: Component = () => {
  const handleStop = () => {
    socket.send("stop");
  };

  const handleStart = () => {
    socket.send("start");
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
      <div class="flex gap-3">
        <button onClick={handleStop}>Stop</button>
        <button onClick={handleStart}>Start</button>
      </div>
      <div ref={pixiContainer}></div>
    </>
  );
};

export default App;
