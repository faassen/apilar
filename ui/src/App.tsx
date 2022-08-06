import {
  Component,
  onCleanup,
  onMount,
  createSignal,
  createEffect,
  createMemo,
} from "solid-js";

import { World, Location } from "./world";
import {
  renderWorld,
  updateWorld,
  getWorldDimensions,
  FillScheme,
  fillSchemes,
  RenderData,
} from "./renderWorld";
import RenderCanvas from "./RenderCanvas";
import Sidebar from "./Sidebar";
import { Viewport } from "pixi-viewport";

const socket = new WebSocket("ws://localhost:3000/ws");

const App: Component = () => {
  const [world, setWorld] = createSignal<World | undefined>();
  const [code, setCode] = createSignal<string | undefined>();
  const [codeError, setCodeError] = createSignal<string | undefined>();
  const [islandId, setIslandId] = createSignal<number>(0);
  const [fillScheme, setFillScheme] = createSignal<FillScheme>(
    fillSchemes.default
  );

  const handleStop = () => {
    socket.send("stop");
  };

  const handleStart = () => {
    socket.send("start");
  };

  let handleWorldUpdate = (event: MessageEvent) => {
    setWorld(JSON.parse(event.data));
  };

  onMount(() => {
    socket.addEventListener("message", handleWorldUpdate);
  });

  onCleanup(() => {
    socket.removeEventListener("message", handleWorldUpdate);
  });

  const handleClick = async ({
    x,
    y,
  }: {
    location: Location;
    x: number;
    y: number;
  }) => {
    const response = await fetch(`/api/disassemble?x=${x}&y=${y}`, {
      method: "GET",
    });
    const json = await response.json();
    if (json.Success != null) {
      setCode(json.Success.code);
      setCodeError(undefined);
    } else {
      setCode(undefined);
      setCodeError(json.Failure.message);
    }
  };
  // browser resize handlers

  createEffect(() => {
    fetch(`/api/observe?island_id=${islandId()}`, {
      method: "POST",
    });
  });

  const renderWorldWithFill = createMemo(() => {
    const currentGetFill = fillScheme();
    return (
      viewport: Viewport,
      world: World,
      onClick: (options: { location: Location; x: number; y: number }) => void
    ) => {
      return renderWorld(viewport, world, currentGetFill, onClick);
    };
  });
  const updateWorldWithFill = createMemo(() => {
    const currentGetFill = fillScheme();
    return (world: World, renderData: RenderData) => {
      return updateWorld(world, renderData, currentGetFill);
    };
  });

  return (
    <div class="flex h-screen flex-col">
      <div class="shrink flex-grow-0 basis-auto flex gap-3">
        <button onClick={handleStop}>Stop</button>
        <button onClick={handleStart}>Start</button>
      </div>
      <div class="shrink flex-grow basis-auto overflow-y-auto flex flex-row w-full">
        <div class="w-4/6">
          <RenderCanvas
            render={renderWorldWithFill()}
            update={updateWorldWithFill()}
            data={world}
            getDimensions={getWorldDimensions}
            onClick={handleClick}
          />
        </div>
        <div class="w-2/6">
          <Sidebar
            world={world}
            islandId={islandId}
            setIslandId={setIslandId}
            fillScheme={fillScheme}
            setFillScheme={setFillScheme}
            code={code}
            codeError={codeError}
          />
        </div>
      </div>
    </div>
  );
};

export default App;
