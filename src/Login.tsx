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
        <div data-tauri-drag-region className="drag-region">
            <div className="wrapper">
                <div className="login-modal">
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
                        {/* <div className="d-flex justify-content-between align-items-center mb-3">
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
                        </div> */}
                        <button
                            type="submit"
                            className="btn btn-primary-custom"
                        >
                            Login
                        </button>
                    </form>
                </div>
            </div>
        </div>
    );
};

export default Login;

// <Helmet>
//     <meta
//         name="viewport"
//         content="width=device-width, initial-scale=1.0"
//     />
//     <title>Login Modal</title>
//     <link
//         href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0-alpha3/dist/css/bootstrap.min.css"
//         rel="stylesheet"
//     />
//     <script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0-alpha3/dist/js/bootstrap.bundle.min.js"></script>
// </Helmet>
