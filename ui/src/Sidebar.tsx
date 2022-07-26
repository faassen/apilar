import { Accessor, Setter, Show } from "solid-js";

import { World } from "./world";
import { FillScheme } from "./fillScheme";
import IslandSelect from "./IslandSelect";
import FillSchemeSelect from "./FillSchemeSelect";

function Sidebar(props: {
  world: Accessor<World | undefined>;
  islandId: Accessor<number>;
  setIslandId: Setter<number>;
  fillScheme: Accessor<FillScheme>;
  setFillScheme: Setter<FillScheme>;
  code: Accessor<string | undefined>;
  codeError: Accessor<string | undefined>;
}) {
  return (
    <Show when={props.world() != null}>
      <div class="flex flex-row gap-3">
        <IslandSelect
          world={props.world as Accessor<World>}
          islandId={props.islandId}
          setIslandId={props.setIslandId}
        />
        <FillSchemeSelect
          fillScheme={props.fillScheme}
          setFillScheme={props.setFillScheme}
        />
      </div>
      <Info
        world={props.world as Accessor<World>}
        islandId={props.islandId}
        code={props.code}
        codeError={props.codeError}
      />
    </Show>
  );
}

function Info(props: {
  world: Accessor<World>;
  islandId: Accessor<number>;
  code: Accessor<string | undefined>;
  codeError: Accessor<string | undefined>;
}) {
  const island = () => {
    return props.world().islands[props.islandId()];
  };
  const processorsPerComputer = () =>
    island().totalProcessors / island().totalComputers;
  const resourcesPerComputer = () =>
    island().totalBoundResources / island().totalComputers;
  const memoryPerComputer = () =>
    island().totalMemoryResources / island().totalComputers;
  const totalResources = () =>
    island().totalFreeResources +
    island().totalBoundResources +
    island().totalMemoryResources;

  return (
    <div class="flex h-full flex-col">
      <div class="shrink flex-grow-0 basis-auto">
        <div>Computers: {island().totalComputers}</div>
        <div>Processors: {island().totalProcessors}</div>
        <div>Processors per computer: {processorsPerComputer().toFixed(3)}</div>
        <div>Resources per computer: {resourcesPerComputer().toFixed(3)}</div>
        <div>Memory per computer: {memoryPerComputer().toFixed(3)}</div>
        <div>Resources Free: {island().totalFreeResources}</div>
        <div>Resources Bound: {island().totalBoundResources}</div>
        <div>Resources Memory: {island().totalMemoryResources}</div>
        <div>Resources total: {totalResources()}</div>
      </div>
      <div class="shrink flex-grow basis-auto overflow-y-auto border">
        <div>Length: {props.code()?.split("\n").length}</div>
        <code>
          <pre>{props.code()}</pre>
        </code>
      </div>
    </div>
  );
}

export default Sidebar;
