import React from "react";
import ReactDOM from "react-dom/client";
import "bootstrap/dist/css/bootstrap.css";
import "./App.css";

import Root from "./App";
import Login from "./Login";
import { createBrowserRouter, RouterProvider } from "react-router-dom";
import Settings from "./Settings";
import TimeCard from "./Timecard";
import Track from "./Track";
import PermissionRequest from "./Permissions";

const router = createBrowserRouter([
  {
    path: "/",
    element: <Root />,
  },
  {
    path: "/login",
    element: <Login />,
  },
  {
    path: "/settings",
    element: <Settings />,
  },
  {
    path: "/timecard",
    element: <TimeCard />,
  },
  {
    path: "/track",
    element: <Track />,
  },
  {
    path: "/permissions",
    element: <PermissionRequest />,
  },
]);

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <RouterProvider router={router} />
  </React.StrictMode>,
);
