import { Component, onCleanup, onMount, createSignal } from "solid-js";

import { World } from "./world";
import { renderWorld, updateWorld, getWorldDimensions } from "./canvas";
import RenderCanvas from "./RenderCanvas";

const socket = new WebSocket("ws://localhost:3000/ws");

const App: Component = () => {
  const [world, setWorld] = createSignal<World | undefined>();

  const handleStop = () => {
    socket.send("stop");
  };

  const handleStart = () => {
    socket.send("start");
  };

  let handleWorldUpdate = (event: MessageEvent) => {
    const world: World = JSON.parse(event.data);
    setWorld(world);
  };

  onMount(() => {
    socket.addEventListener("message", handleWorldUpdate);
  });

  onCleanup(() => {
    socket.removeEventListener("message", handleWorldUpdate);
  });

  // browser resize handlers

  return (
    <div class="h-screen">
      <div class="flex gap-3">
        <button onClick={handleStop}>Stop</button>
        <button onClick={handleStart}>Start</button>
      </div>
      <div class="w-full h-5/6">
        <RenderCanvas
          render={renderWorld}
          update={updateWorld}
          data={world}
          getDimensions={getWorldDimensions}
        />
      </div>
    </div>
  );
};

export default App;
