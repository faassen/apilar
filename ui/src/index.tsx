/* @refresh reload */
import { render } from "solid-js/web";
import "tailwindcss/tailwind.css";

import "./index.css";

import App from "./App";

const socket = new WebSocket("ws://localhost:3000/ws");

// socket.addEventListener("open", function (event) {
//   socket.send("Hello Server!");
// });

socket.addEventListener("message", (event) => {
  console.log("Message from server ", event.data);
});

render(() => <App />, document.getElementById("root") as HTMLElement);
