import "./styles/Login.scss";
import reactLogo from "./assets/react.svg";
import { FormEventHandler } from "react";
import { login } from "./ipc";

const Login = () => {
  const onSubmit: FormEventHandler = async (evt) => {
    evt.preventDefault();
    const form = evt.target as HTMLFormElement;
    var formData = new FormData(form);
    // output as an object
    console.log(formData.get("email"), formData.get("password"));
    await login();
  };

  return (
    // <div className="drag-region login-modal" data-tauri-drag-region>
    //   <button className="btn-close-custom" aria-label="Close">
    //     ×
    //   </button>
    //   <img src={reactLogo} style={{ marginBottom: "35px" }} />
    //   <form onSubmit={onSubmit}>
    //     <div className="mb-3">
    //       <label htmlFor="email" className="form-label">
    //         Email
    //       </label>
    //       <input
    //         type="email"
    //         name="email"
    //         className="form-control"
    //         id="email"
    //         placeholder="Email"
    //         required
    //       />
    //     </div>
    //     <div className="mb-3">
    //       <label htmlFor="password" className="form-label">
    //         Password
    //       </label>
    //       <input
    //         type="password"
    //         name="password"
    //         className="form-control"
    //         id="password"
    //         placeholder="Password"
    //         required
    //       />
    //     </div>

    //     <button type="submit" className="btn btn-primary-custom">
    //       Login
    //     </button>
    //   </form>
    // </div>
    <div className="login-card" data-tauri-drag-region>
      <button className="btn-close-custom" aria-label="Close">
        ×
      </button>
      <img src={reactLogo} style={{ marginBottom: "35px" }} />
      <form onSubmit={onSubmit}>
        <div className="mb-3">
          <label htmlFor="email" className="form-label">
            Email
          </label>
          <input
            type="email"
            name="email"
            className="form-control"
            id="email"
            placeholder="Email"
            required
          />
        </div>
        <div className="mb-3">
          <label htmlFor="password" className="form-label">
            Password
          </label>
          <input
            type="password"
            name="password"
            className="form-control"
            id="password"
            placeholder="Password"
            required
          />
        </div>

        <button type="submit" className="btn btn-primary-custom">
          Login
        </button>
      </form>
    </div>
  );
};

export default Login;

{
  /* <div className="d-flex justify-content-between align-items-center mb-3">
                    <div className="form-check">
                        <input
                            type="checkbox"
                            className="form-check-input"
                            id="rememberMe"
                        />
                        <label
                            className="form-check-label"
                            htmlFor="rememberMe"
                        >
                            Remember me
                        </label>
                    </div>
                    <a href="#" className="text-muted">
                        Forgot password?
                    </a>
                </div> */
}
