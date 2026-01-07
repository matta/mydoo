import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { renderWithTestProviders } from "../../test/setup";
import { DateInput } from "./date-input";

describe("DateInput", () => {
  it("renders with a label", () => {
    renderWithTestProviders(
      <DateInput label="Due Date" value={null} onChange={() => {}} />,
    );
    expect(screen.getByText("Due Date")).toBeInTheDocument();
  });

  it("displays a date value in YYYY-MM-DD format", () => {
    const testDate = new Date(2026, 0, 15); // Jan 15, 2026
    renderWithTestProviders(
      <DateInput label="Due Date" value={testDate} onChange={() => {}} />,
    );

    const input = screen.getByTestId("date-input");
    expect(input).toHaveValue("2026-01-15");
  });

  it("displays empty string when value is null", () => {
    renderWithTestProviders(
      <DateInput label="Due Date" value={null} onChange={() => {}} />,
    );

    const input = screen.getByTestId("date-input");
    expect(input).toHaveValue("");
  });

  it("calls onChange with a Date when user selects a date", async () => {
    const user = userEvent.setup();
    const handleChange = vi.fn();
    renderWithTestProviders(
      <DateInput label="Due Date" value={null} onChange={handleChange} />,
    );

    const input = screen.getByTestId("date-input");
    await user.clear(input);
    await user.type(input, "2026-03-20");

    expect(handleChange).toHaveBeenCalled();
    const resultDate = handleChange.mock.lastCall?.[0] as Date;
    expect(resultDate.getFullYear()).toBe(2026);
    expect(resultDate.getMonth()).toBe(2); // March = 2
    expect(resultDate.getDate()).toBe(20);
  });

  it("calls onChange with null when user clears the date", async () => {
    const user = userEvent.setup();
    const handleChange = vi.fn();
    const testDate = new Date(2026, 0, 15);
    renderWithTestProviders(
      <DateInput label="Due Date" value={testDate} onChange={handleChange} />,
    );

    const input = screen.getByTestId("date-input");
    await user.clear(input);

    expect(handleChange).toHaveBeenCalledWith(null);
  });
});
