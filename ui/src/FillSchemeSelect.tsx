import { Accessor, Setter, For } from "solid-js";
import { fillSchemes, FillScheme } from "./fillScheme";

function FillSchemeSelect(props: {
  fillScheme: Accessor<FillScheme>;
  setFillScheme: Setter<FillScheme>;
}) {
  return (
    <select
      onChange={(e) =>
        props.setFillScheme(() => fillSchemes[e.currentTarget.value])
      }
    >
      <For each={Object.entries(fillSchemes)}>
        {([fillSchemeId, fillScheme]) => (
          <option
            selected={fillScheme === props.fillScheme()}
            value={fillSchemeId}
          >
            {fillSchemeId.toString().padStart(15, " ")}
          </option>
        )}
      </For>
    </select>
  );
}

export default FillSchemeSelect;
