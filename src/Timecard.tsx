import "./styles/TimeCard.css";

const TimeCard = () => {
  return (
    <div data-tauri-drag-region className="timecard">
      <button data-tauri-drag-region>
        <i className="bi bi-arrow-left-short"></i>
      </button>
      <div className="details" data-tauri-drag-region>
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
  );
};

export default TimeCard;
