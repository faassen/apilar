import { Component, onCleanup, onMount } from "solid-js";
import Konva from "konva";

import {
  render,
  World,
  Location,
  updateWorldLayer,
  WorldShapes,
  BOX_SIZE,
} from "./canvas";

const App: Component = () => {
  const world: World = {
    width: 200,
    height: 200,
    locations: [],
  };

  for (let y = 0; y < world.height; y++) {
    let row: Location[] = [];
    for (let x = 0; x < world.width; x++) {
      row.push({
        freeResources: 100,
        computer: null,
      });
    }
    world.locations.push(row);
  }

  let stage: Konva.Stage;
  let layer: Konva.Layer;
  let worldShapes: WorldShapes;

  // padding will increase the size of stage
  // so scrolling will look smoother
  const PADDING = 1000;

  let scrollContainer: HTMLDivElement | undefined;
  const repositionStage = () => {
    if (scrollContainer == null) {
      return;
    }
    var dx = scrollContainer.scrollLeft - PADDING;
    var dy = scrollContainer.scrollTop - PADDING;
    stage.container().style.transform = "translate(" + dx + "px, " + dy + "px)";
    stage.x(-dx);
    stage.y(-dy);
  };

  onMount(() => {
    stage = new Konva.Stage({
      container: "canvas",
      width: world.width * BOX_SIZE,
      height: world.height * BOX_SIZE,
    });
    [layer, worldShapes] = render(stage, world);
    scrollContainer?.addEventListener("scroll", repositionStage);
    repositionStage();
  });

  onCleanup(() => {
    scrollContainer?.removeEventListener("scroll", repositionStage);
  });

  const handleUpdate = () => {
    console.log("updating world");
    for (let iy = 10; iy < 20; iy++) {
      for (let ix = 10; ix < 20; ix++) {
        world.locations[iy][ix].computer = {
          memorySize: 3,
          boundResources: 10,
          processors: 5,
        };
      }
    }
    updateWorldLayer(layer, world, worldShapes);
  };

  return (
    <>
      <button onClick={handleUpdate}>Update</button>
      <div id="scroll-container" ref={scrollContainer}>
        <div
          id="large-container"
          class="overflow-hidden"
          style={{
            width: `${world.width * BOX_SIZE}px`,
            height: `${world.height * BOX_SIZE}px`,
          }}
        >
          <div id="canvas"></div>
        </div>
      </div>
    </>
  );
};

export default App;
