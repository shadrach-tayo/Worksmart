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

const Track = () => {
    return (
        <div data-tauri-drag-region className="tracker-card">
            <div className="d-flex flex-column gap-3">
                <div className="tracker-header">
                    <h2>Logo</h2>
                    <div className="tab-control">
                        {/* <i className="bi bi-dash-lg"></i> */}
                        <Minus />
                        <X />
                        {/* <i className="bi bi-x-lg"></i> */}
                    </div>
                </div>
                <div className="divider"></div>
                <div className="d-flex justify-content-between">
                    <div className="tracker-details">
                        {/* <i className="bi bi-circle-fill"></i> */}
                        <Circle fill="#3cbd0f" stroke="transparent" size={14} />
                        <div>
                            <h4>Shadrach Tayo</h4>
                            <h5>3h 50m</h5>
                        </div>
                    </div>
                    <button className="action-button start-button">
                        <Play fill="#3cbd0f" stroke="transparent" />
                    </button>
                    <button className="action-button stop-button">
                        <Square fill="red" stroke="transparent" />
                    </button>
                </div>
            </div>
            <div className="tracker-menu">
                <Settings />
                <ChartPie />
            </div>
        </div>
    );
};

export default Track;
