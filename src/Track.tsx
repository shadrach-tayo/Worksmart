import { useEffect, useRef, useState } from "react";
import "./styles/Track.css";
import {
  X,
  Minus,
  ChartPie,
  Settings,
  Square,
  Circle,
  Play,
} from "lucide-react";
import { Session, User } from "./types";
import {
  get_session,
  get_time_tracked_today,
  get_user,
  minimize_window,
  quit_app,
  show_window,
  start_session,
  stop_session,
} from "./ipc";
import { open } from "@tauri-apps/api/shell";
import { listen } from "@tauri-apps/api/event";
import { relativeTime } from "./helper";

const Track = () => {
  const [user, setUser] = useState<User>();
  const [session, setSession] = useState<Session>();
  const intervalRef = useRef<number>();
  const eventRef = useRef<Function>();
  const [timeTrackedToday, setTimeTrackedToday] = useState<string>();

  let isActive = session !== undefined && !session?.ended_at;
  let isEnded = session !== undefined && session?.ended_at;

  const startSession = async () => {
    let session = await start_session();
    // console.log("start", session);
    setSession(session);
  };

  const stopSession = async () => {
    await stop_session();
    // setSession(undefined);
    const session = await get_session();
    session && setSession(session);
  };

  const pullData = async () => {
    const user = await get_user();
    const session = await get_session();

    console.log("pullData", { session, user });

    user && setUser(user);
    session && setSession(session);
  };

  const listenForEvent = async () => {
    eventRef.current = await listen("SessionEnded", pullData);
  };

  useEffect(() => {
    pullData();
    listenForEvent();
    return () => eventRef.current?.();
  }, []);

  const pollData = async () => {
    const timestamp = await get_time_tracked_today();
    setTimeTrackedToday(relativeTime(timestamp));
  };

  useEffect(() => {
    intervalRef.current = setInterval(pollData, 30);
    return () => clearInterval(intervalRef.current);
  }, []);

  // console.log("session", session);
  return (
    <div data-tauri-drag-region className="tracker-card">
      <div className="d-flex flex-column gap-3">
        <div className="tracker-header">
          <h2>Worksmart</h2>
          <div className="tab-control">
            <button
              className="icon-btn"
              onClick={() => minimize_window("track")}
            >
              <Minus />
            </button>
            <button className="icon-btn" onClick={quit_app} disabled={isActive}>
              <X />
            </button>
          </div>
        </div>
        <div className="divider"></div>
        <div>
          <div className="d-flex justify-content-between">
            <div className="tracker-details">
              <Circle
                fill={isActive ? "red" : "#3cbd0f"}
                stroke="transparent"
                size={14}
              />
              <div>
                <h4>{user?.name}</h4>
                {/* <Timer start="2024-10-06T23:30:49.786Z" />
                              <Timer start="2024-10-06T05:00:00.000Z" /> */}
                {isActive ? (
                  <Timer start={session?.started_at!} />
                ) : isEnded ? (
                  <Timer start={session?.started_at!} end={session?.ended_at} />
                ) : null}
              </div>
            </div>
            {isActive ? (
              <button
                className="action-button stop-button"
                onClick={stopSession}
              >
                <Square fill="red" stroke="transparent" size={20} />
              </button>
            ) : (
              <button
                className="action-button start-button"
                onClick={startSession}
              >
                <Play fill="#3cbd0f" stroke="transparent" size={28} />
              </button>
            )}
          </div>
          {timeTrackedToday && (
            <div className="tracked-time">
              <Circle fill={"#3cbd0f"} stroke="transparent" size={12} />
              <p>{timeTrackedToday}</p>
            </div>
          )}
        </div>
      </div>
      <div className="tracker-menu">
        <button className="icon-btn" onClick={() => show_window("settings")}>
          <Settings />
        </button>
        <button
          className="icon-btn"
          onClick={() => open("https://shadrachtayo.com")}
        >
          <ChartPie />
        </button>
      </div>
    </div>
  );
};

const Timer = (props: { start: string; end?: string }) => {
  const [{ hour, minute }, setClock] = useState<{
    hour: number;
    minute: number;
  }>({ hour: 0, minute: 0 });

  const getTimeDiff = (date: string) => {
    let hour = 0;
    let minute = 0;

    let now = props.end ? new Date(props.end).getTime() : new Date().getTime();
    let diff = (now - new Date(date).getTime()) / 1000;
    hour = diff / (60 * 60);
    minute = hour >= 1 ? Math.floor((hour % 1) * 60) : Math.floor(diff / 60);

    return { hour: Math.floor(hour), minute };
  };

  useEffect(() => {
    const tick = () => {
      const clock = getTimeDiff(props.start);
      setClock(clock);
    };
    let timerRef = props.start ? setInterval(tick, 1000) : undefined;

    return () => clearInterval(timerRef);
  }, [props]);

  return (
    <p className="timer">
      {hour ? `${hour}h` : ""} {`${minute}m`}
    </p>
  );
};

export default Track;
