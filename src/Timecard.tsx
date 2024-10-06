import "./styles/TimeCard.css"; // Assuming styles are moved to a separate CSS file named TimeCard.css

const TimeCard = () => {
    return (
        <div data-tauri-drag-region className="drag-region">
            <div className="timecard">
                <button className="data-tauri-drag-region">
                    <i className="bi bi-arrow-left-short"></i>
                </button>
                <div className="details">
                    <div className="d-flex justify-content-between">
                        <p>Timecards to upload</p>
                        <p>0m</p>
                    </div>
                    <div className="d-flex justify-content-between">
                        <p>Last timecard upload</p>
                        <p>13:15</p>
                    </div>
                    <div className="d-flex justify-content-between">
                        <p>Last server sync</p>
                        <p>13:17</p>
                    </div>
                </div>
            </div>
        </div>
    );
};

export default TimeCard;
