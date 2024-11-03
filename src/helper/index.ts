export function relativeTime(timestamp: number) {
  const diff = timestamp; //Math.round((new Date().getTime() - new Date(date).getTime()) / 1000);

  const minute = 60;
  const hour = minute * 60;

  // Calculate the number of hours and leftover minutes
  const hours = Math.floor(diff / hour);
  const minutes = Math.floor((diff % hour) / minute);

  // Format the result as 'HH:MM' or 'MM' if there are no hours.
  let formattedTime = "";

  if (hours > 0) {
    formattedTime +=
      (hours < 10 ? "0" + hours : hours) + ` Hour${hours > 1 ? "s" : ""} `;
    formattedTime +=
      minutes < 10
        ? "0" + minutes
        : minutes + ` Minute${minutes > 1 ? "s" : ""}`;
  } else if (minutes > 0) {
    formattedTime += "" + minutes + ` Minute${minutes > 1 ? "s" : ""}`;
  } else {
    formattedTime = "Less than a minute";
  }

  return formattedTime;
}
//
