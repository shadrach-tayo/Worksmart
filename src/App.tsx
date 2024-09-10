import { useState } from "react";
// import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";

function App() {
  const [_greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    setGreetMsg(await invoke("greet", { name }));
  }

  async function capture_screen() {
    await invoke("capture_screen");
  }

  return (
    <div className="container">
      <h1>Worksmart ⚡️</h1>

      <p>Start work session ⏰</p>
      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter team ID..."
        />
        <button type="submit">Start session</button>
      </form>
      <div
        className="my-2 flex items-center justify-center"
        style={{ margin: "8px 0" }}
      >
        <button onClick={capture_screen}>Capture Screen</button>
      </div>
    </div>
  );
}

export default App;
