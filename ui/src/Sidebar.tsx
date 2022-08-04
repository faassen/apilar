import { Accessor, Show } from "solid-js";

import { World } from "./world";

function Sidebar(props: {
  world: Accessor<World | undefined>;
  code: Accessor<string | undefined>;
  codeError: Accessor<string | undefined>;
}) {
  return (
    <Show when={props.world() != null}>
      <Info
        world={props.world as Accessor<World>}
        code={props.code}
        codeError={props.codeError}
      />
    </Show>
  );
}

function Info(props: {
  world: Accessor<World>;
  code: Accessor<string | undefined>;
  codeError: Accessor<string | undefined>;
}) {
  const processorsPerComputer = () =>
    props.world().totalProcessors / props.world().totalComputers;
  const resourcesPerComputer = () =>
    props.world().totalBoundResources / props.world().totalComputers;
  const memoryPerComputer = () =>
    props.world().totalMemoryResources / props.world().totalComputers;
  const totalResources = () =>
    props.world().totalFreeResources +
    props.world().totalBoundResources +
    props.world().totalMemoryResources;

  return (
    <div class="flex h-full flex-col">
      <div class="shrink flex-grow-0 basis-auto">
        <div>Computers: {props.world().totalComputers}</div>
        <div>Processors: {props.world().totalProcessors}</div>
        <div>Processors per computer: {processorsPerComputer().toFixed(3)}</div>
        <div>Resources per computer: {resourcesPerComputer().toFixed(3)}</div>
        <div>Memory per computer: {memoryPerComputer().toFixed(3)}</div>
        <div>Resources Free: {props.world().totalFreeResources}</div>
        <div>Resources Bound: {props.world().totalBoundResources}</div>
        <div>Resources Memory: {props.world().totalMemoryResources}</div>
        <div>Resources total: {totalResources()}</div>
      </div>
      <div class="shrink flex-grow basis-auto overflow-y-auto border">
        <code>
          <pre>{props.code()}</pre>
        </code>
      </div>
    </div>
  );
}

export default Sidebar;
