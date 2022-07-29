import { Component, onCleanup, onMount } from "solid-js";

import * as pixi from "pixi.js";
import { Viewport } from "pixi-viewport";

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

  const screenWidth = window.innerWidth;
  const screenHeight = window.innerHeight - 100;

  // application width & height needs to be the same as viewport
  // width and height otherwise we can't fully scroll to the right somehow
  let app = new pixi.Application({
    width: screenWidth,
    height: screenHeight,
    backgroundAlpha: 0,
    resolution: window.devicePixelRatio,
  });

  const createViewport = (width: number, height: number) => {
    const viewport = new Viewport({
      screenWidth: screenWidth,
      screenHeight: screenHeight,
      worldWidth: width * BOX_SIZE,
      worldHeight: height * BOX_SIZE,
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

  let worldShapes: WorldShapes | undefined;

  let handleWorldUpdate = (event: MessageEvent) => {
    const world: World = JSON.parse(event.data);

    if (worldShapes == null) {
      const viewport = createViewport(world.width, world.height);
      worldShapes = renderWorld(viewport, world);
    } else {
      updateWorld(world, worldShapes);
    }
  };

  onMount(() => {
    pixiContainer?.appendChild(app.view);

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
