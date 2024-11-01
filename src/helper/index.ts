export function relativeTime(timestamp: number) {
  const diff = timestamp; //Math.round((new Date().getTime() - new Date(date).getTime()) / 1000);

  const minute = 60;
  const hour = minute * 60;
  const day = hour * 24;
  const week = day * 7;
  const month = day * 30;
  const year = month * 12;

  if (diff < 30) {
    return "just now";
  } else if (diff < minute) {
    return diff + " seconds";
  } else if (diff < 2 * minute) {
    return "A minutes";
  } else if (diff < hour) {
    const sub = Math.floor(diff / minute);
    return sub + ` minute${sub > 1 ? "s" : ""}`;
  } else if (Math.floor(diff / hour) == 1) {
    return "An hour";
  } else if (diff < day) {
    const sub = Math.floor(diff / hour);
    return sub + ` hour${sub > 1 ? "s" : ""}`;
  } else if (diff < day * 2) {
    return "yesterday";
  } else if (diff < week) {
    const sub = Math.floor(diff / day);
    return sub + " days";
  } else if (diff < month) {
    const sub = Math.floor(diff / week);
    return `${sub === 1 ? "A" : sub} week${sub > 1 ? "s" : ""}`;
  } else if (diff < year) {
    const sub = Math.floor(diff / month);
    return `${sub === 1 ? "A" : sub} month${sub > 1 ? "s" : ""}`;
  } else {
    const sub = Math.floor(diff / year);
    return `${sub === 1 ? "A" : sub} year${sub > 1 ? "s" : ""}`;
  }
}
