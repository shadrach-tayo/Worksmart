import { useEffect, useState } from "react";
// import reactLogo from "./assets/react.svg";
import "./App.css";
import {
    check_permissions,
    record_screen,
    request_permissions,
    start_session,
    stop_session,
    update_config,
    webcam_capture,
} from "./ipc";

function App() {
    const [hasPermission, setHasPermission] = useState(false);

    useEffect(() => {
        const check = async () => {
            setHasPermission(await check_permissions());
        };

        check();
    }, []);

    return (
        <div className="container">
            <h1>Worksmart ⚡️</h1>

            <p>Start work session ⏰</p>
            {/* <form
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
      </form> */}
            <div
                className="my-2 flex items-center justify-between gap-3"
                style={{
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "space-between",
                    gap: "5px",
                    maxWidth: "700px",
                    margin: "10px auto",
                }}
            >
                {hasPermission ? (
                    <>
                        <button onClick={start_session}>Start session</button>
                        <button onClick={stop_session}>Stop session</button>
                        <button onClick={record_screen}>Record Screen</button>
                        <button onClick={update_config}>Update config</button>
                        <button onClick={webcam_capture}>
                            Take Webcam shot
                        </button>
                    </>
                ) : (
                    <div>
                        <p>
                            You need to grant worksmart permissions to record
                            and capture your screens and webcam
                        </p>
                        <button onClick={request_permissions}>
                            Request permissions
                        </button>
                    </div>
                )}
            </div>
        </div>
    );
}

export default App;
