export type CalendarMode = 'yeardays_only' | 'monthdays_weekdays';

const MONTHS = [
  { name: 'January', days: 31 },
  { name: 'February', days: 28 },
  { name: 'March', days: 31 },
  { name: 'April', days: 30 },
  { name: 'May', days: 31 },
  { name: 'June', days: 30 },
  { name: 'July', days: 31 },
  { name: 'August', days: 31 },
  { name: 'September', days: 30 },
  { name: 'October', days: 31 },
  { name: 'November', days: 30 },
  { name: 'December', days: 31 },
] as const;

const WEEKDAYS = ['Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday', 'Sunday'] as const;

export interface CalendarDate {
  year: number;
  dayOfYear: number;
  month: number;
  dayOfMonth: number;
  monthName: string;
  weekdayName: string;
}

export function calendarMode(labels: Record<string, string>): CalendarMode {
  return labels.calendar_mode === 'monthdays_weekdays' ? 'monthdays_weekdays' : 'yeardays_only';
}

export function calendarDate(day: number, daysPerYear: number, mode: CalendarMode): CalendarDate {
  const safeDay = Math.max(1, day);
  const year = Math.floor((safeDay - 1) / daysPerYear) + 1;
  const dayOfYear = ((safeDay - 1) % daysPerYear) + 1;
  let remaining = dayOfYear;
  let month = 1;
  while (month <= MONTHS.length && remaining > MONTHS[month - 1].days) {
    remaining -= MONTHS[month - 1].days;
    month += 1;
  }
  const monthInfo = MONTHS[Math.min(month, MONTHS.length) - 1];
  return {
    year,
    dayOfYear,
    month,
    dayOfMonth: mode === 'monthdays_weekdays' ? remaining : dayOfYear,
    monthName: mode === 'monthdays_weekdays' ? monthInfo.name : '',
    weekdayName: WEEKDAYS[(safeDay - 1) % WEEKDAYS.length],
  };
}

export function formatCalendarDay(day: number, daysPerYear: number, labels: Record<string, string>): string {
  const mode = calendarMode(labels);
  const date = calendarDate(day, daysPerYear, mode);
  if (mode === 'monthdays_weekdays') {
    return `${date.weekdayName}, ${date.dayOfMonth}.${date.monthName}, Year ${date.year}`;
  }
  return `Year ${date.year}, Day ${date.dayOfYear}`;
}

export function formatEventDate(dayOfYear: number, daysPerYear: number, labels: Record<string, string>): string {
  if (calendarMode(labels) !== 'monthdays_weekdays') return `Day ${dayOfYear}`;
  const date = calendarDate(dayOfYear, daysPerYear, 'monthdays_weekdays');
  return `${date.dayOfMonth}.${date.monthName}`;
}
