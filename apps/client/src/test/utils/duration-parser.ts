/**
 * Parsed duration with value and unit information.
 */
export interface ParsedDuration {
  /** Numeric value (e.g., 3 from "3 days") */
  value: number;
  /** Raw unit string (e.g., "days", "hour") */
  rawUnit: string;
  /** UI-friendly unit label (e.g., "Days", "Hours", "Minutes") */
  uiUnit: 'Days' | 'Hours' | 'Minutes';
}

/**
 * Parses a human-readable duration string into structured components.
 * Throws an error if the format is invalid or unrecognized.
 *
 * @example
 * parseDuration("3 days")  // { value: 3, rawUnit: "days", uiUnit: "Days" }
 * parseDuration("1 hour")  // { value: 1, rawUnit: "hour", uiUnit: "Hours" }
 *
 * @throws {Error} If the string format is invalid or unit is unrecognized
 */
export function parseDuration(str: string): ParsedDuration {
  const parts = str.trim().split(/\s+/);

  if (parts.length !== 2) {
    throw new Error(
      `Invalid duration format: "${str}". Expected format: "<number> <unit>" (e.g., "3 days")`,
    );
  }

  const valueStr = parts[0];
  const unitStr = parts[1];

  // These checks should be unreachable after the length check, but satisfy TypeScript
  if (valueStr === undefined || unitStr === undefined) {
    throw new Error(`Invalid duration format: "${str}"`);
  }

  const rawUnit = unitStr.toLowerCase();
  const value = parseFloat(valueStr);

  if (Number.isNaN(value)) {
    throw new Error(
      `Invalid duration value: "${valueStr}" in "${str}". Expected a number.`,
    );
  }

  // Explicit allowlists for valid unit strings
  const minuteUnits = ['minute', 'minutes', 'min', 'mins'];
  const hourUnits = ['hour', 'hours', 'hr', 'hrs'];
  const dayUnits = ['day', 'days'];

  let uiUnit: ParsedDuration['uiUnit'];
  if (minuteUnits.includes(rawUnit)) {
    uiUnit = 'Minutes';
  } else if (hourUnits.includes(rawUnit)) {
    uiUnit = 'Hours';
  } else if (dayUnits.includes(rawUnit)) {
    uiUnit = 'Days';
  } else {
    throw new Error(
      `Unrecognized duration unit: "${rawUnit}" in "${str}". Valid units: ${[...minuteUnits, ...hourUnits, ...dayUnits].join(', ')}`,
    );
  }

  return { value, rawUnit, uiUnit };
}

/**
 * Converts a duration string to milliseconds.
 * Throws an error if the format is invalid or unrecognized.
 *
 * @example
 * durationToMs("3 days")  // 259200000 (3 * 24 * 60 * 60 * 1000)
 * durationToMs("2 hours") // 7200000 (2 * 60 * 60 * 1000)
 *
 * @throws {Error} If the string format is invalid or unit is unrecognized
 */
export function durationToMs(str: string): number {
  const { value, uiUnit } = parseDuration(str);

  // Use validated uiUnit field for conversion
  switch (uiUnit) {
    case 'Minutes':
      return value * 60 * 1000;
    case 'Hours':
      return value * 60 * 60 * 1000;
    case 'Days':
      return value * 24 * 60 * 60 * 1000;
  }
}
