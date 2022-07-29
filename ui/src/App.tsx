import { Component, onCleanup, onMount } from "solid-js";

import * as pixi from "pixi.js";
import { Viewport } from "pixi-viewport";
import { Simple, SpatialHash } from "pixi-cull";

import { World, Location } from "./world";
import { renderWorld, updateWorld, BOX_SIZE, WorldShapes } from "./canvas";
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

  const width = window.innerWidth;
  const height = window.innerHeight - 100;

  // application width & height needs to be the same as viewport
  // width and height otherwise we can't fully scroll to the right somehow
  let app = new pixi.Application({
    width: width,
    height: height,
    backgroundAlpha: 0,
    resolution: window.devicePixelRatio,
  });

  const createViewport = () => {
    const viewport = new Viewport({
      screenWidth: width,
      screenHeight: height,
      worldWidth: 70 * BOX_SIZE,
      worldHeight: 40 * BOX_SIZE,
      // worldWidth: world.width * BOX_SIZE,
      // worldHeight: world.height * BOX_SIZE,
      interaction: app.renderer.plugins.interaction,
    });

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
    return viewport;
  };

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

  let worldShapes: WorldShapes | undefined;

  let handleWorldUpdate = (event: MessageEvent) => {
    const world: World = JSON.parse(event.data);

    if (worldShapes == null) {
      const viewport = createViewport();
      worldShapes = renderWorld(viewport, world);
    } else {
      updateWorld(world, worldShapes);
    }
  };

  onMount(() => {
    pixiContainer?.appendChild(app.view);
    // cull.cull(viewport.getVisibleBounds());

    socket.addEventListener("message", handleWorldUpdate);
  });

  onCleanup(() => {
    socket.removeEventListener("message", handleWorldUpdate);
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
