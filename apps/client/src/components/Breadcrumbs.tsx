import {type TaskID} from '@mydoo/tasklens';
import {type BreadcrumbItem} from '../lib/todoUtils';

/**
 * Props for the Breadcrumbs component.
 */
interface BreadcrumbsProps {
  crumbs: BreadcrumbItem[];
  onNavigate: (path: TaskID[]) => void;
}

import {Breadcrumbs as MantineBreadcrumbs, Anchor, Text} from '@mantine/core';

export function Breadcrumbs({crumbs, onNavigate}: BreadcrumbsProps) {
  const items = crumbs.map((crumb, index) => {
    const isLast = index === crumbs.length - 1;
    const key = crumb.type === 'root' ? 'root' : crumb.id;

    if (isLast) {
      return (
        <Text key={key} c="dimmed">
          {crumb.title}
        </Text>
      );
    }

    return (
      <Anchor
        key={key}
        component="button"
        type="button"
        onClick={() => {
          onNavigate(crumb.path);
        }}
        size="sm"
      >
        {crumb.title}
      </Anchor>
    );
  });

  return (
    <MantineBreadcrumbs separator="→" mt="xs">
      {items}
    </MantineBreadcrumbs>
  );
}
