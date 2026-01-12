import { Badge, type BadgeProps, type MantineColor } from "@mantine/core";
import { getUrgencyStatus, UrgencyStatus } from "@mydoo/tasklens";
import { useMemo } from "react";
import { formatDueDate } from "../../lib/date-formatting";

export interface DueDateBadgeProps extends BadgeProps {
  effectiveDueDate: number | undefined;
  effectiveLeadTime: number | undefined;
}

const STATUS_COLORS: Record<UrgencyStatus, MantineColor | undefined> = {
  [UrgencyStatus.Overdue]: "red",
  [UrgencyStatus.Urgent]: "orange",
  [UrgencyStatus.Active]: "yellow",
  [UrgencyStatus.Upcoming]: "green",
  [UrgencyStatus.None]: undefined,
};

export function DueDateBadge({
  effectiveDueDate,
  effectiveLeadTime,
  ...props
}: DueDateBadgeProps) {
  const status = useMemo(() => {
    return getUrgencyStatus(effectiveDueDate, effectiveLeadTime, Date.now());
  }, [effectiveDueDate, effectiveLeadTime]);

  // If no status or None, render nothing
  if (status === UrgencyStatus.None || effectiveDueDate === undefined) {
    return null;
  }

  const label = formatDueDate(effectiveDueDate);
  const color = STATUS_COLORS[status];

  if (!color) return null;

  return (
    <Badge
      color={color}
      variant="light"
      size="sm"
      data-urgency={status}
      {...props}
    >
      {label}
    </Badge>
  );
}
