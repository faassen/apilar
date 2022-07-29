import { Component, onCleanup, onMount, createSignal } from "solid-js";

import { World } from "./world";
import { renderWorld, updateWorld, getWorldDimensions } from "./renderWorld";
import RenderCanvas from "./RenderCanvas";
import Sidebar from "./Sidebar";

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
      <div class="flex flex-row w-full h-5/6">
        <div class="w-4/6 h-full">
          <RenderCanvas
            render={renderWorld}
            update={updateWorld}
            data={world}
            getDimensions={getWorldDimensions}
          />
        </div>
        <div class="w-2/6">
          <Sidebar world={world} />
        </div>
      </div>
    </div>
  );
};

export default App;
