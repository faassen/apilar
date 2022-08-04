import { Accessor, Setter, For } from "solid-js";

import { World } from "./world";

function IslandSelect(props: {
  world: Accessor<World>;
  islandId: Accessor<number>;
  setIslandId: Setter<number>;
}) {
  return (
    <select onChange={(e) => props.setIslandId(Number(e.currentTarget.value))}>
      <For each={props.world().islands}>
        {(_, id) => (
          <option selected={id() == props.islandId()} value={id().toString()}>
            {id}
          </option>
        )}
      </For>
    </select>
  );
}

export default IslandSelect;
