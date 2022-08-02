import { createEffect, onMount, Accessor } from "solid-js";

import * as pixi from "pixi.js";
import { Viewport } from "pixi-viewport";

function RenderCanvas<T, R, C>(props: {
  data: Accessor<T | undefined>;
  render: (viewport: Viewport, data: T, onClick: (c: C) => void) => R;
  update: (data: T, renderData: R) => void;
  getDimensions: (data: T) => [number, number];
  onClick: (c: C) => void;
}) {
  let pixiContainer: HTMLDivElement | undefined;

  const createViewport = (
    app: pixi.Application,
    width: number,
    height: number
  ) => {
    const viewport = new Viewport({
      screenWidth: app.view.width,
      screenHeight: app.view.height,
      worldWidth: width,
      worldHeight: height,
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

  onMount(() => {
    if (!pixiContainer) {
      return;
    }
    let app = new pixi.Application({
      width: pixiContainer.offsetWidth,
      height: pixiContainer.offsetHeight,
      backgroundAlpha: 0,
      resolution: window.devicePixelRatio,
    });

    pixiContainer.appendChild(app.view);

    let renderData: R | undefined;

    createEffect(() => {
      let data = props.data();
      if (data == null) {
        return;
      }
      const [width, height] = props.getDimensions(data);
      if (renderData == null) {
        const viewport = createViewport(app, width, height);
        renderData = props.render(viewport, data, props.onClick);
      } else {
        props.update(data, renderData);
      }
    });
  });

  return <div class="w-full h-full" ref={pixiContainer}></div>;
}

export default RenderCanvas;
