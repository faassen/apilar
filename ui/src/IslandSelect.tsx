import { Accessor, Setter, Index } from "solid-js";

import { World } from "./world";

function IslandSelect(props: {
  world: Accessor<World>;
  islandId: Accessor<number>;
  setIslandId: Setter<number>;
}) {
  return (
    <select onChange={(e) => props.setIslandId(Number(e.currentTarget.value))}>
      <Index each={props.world().islands}>
        {(_, id) => (
          <option selected={id == props.islandId()} value={id.toString()}>
            {id.toString().padStart(5, "0")}
          </option>
        )}
      </Index>
    </select>
  );
}

export default IslandSelect;
