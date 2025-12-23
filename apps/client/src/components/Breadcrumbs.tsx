import type {TaskID} from '@mydoo/tasklens';

import type {BreadcrumbItem} from '../lib/todoUtils';

/**
 * Props for the Breadcrumbs component.
 */
interface BreadcrumbsProps {
  crumbs: BreadcrumbItem[];
  onNavigate: (path: TaskID[]) => void;
}

import {Anchor, Breadcrumbs as MantineBreadcrumbs, Text} from '@mantine/core';

export function Breadcrumbs({crumbs, onNavigate}: BreadcrumbsProps) {
  const items = crumbs.map((crumb, index) => {
    const isLast = index === crumbs.length - 1;
    const key = crumb.type === 'root' ? 'root' : crumb.id;

    if (isLast) {
      return (
        <Text c="dimmed" key={key}>
          {crumb.title}
        </Text>
      );
    }

    return (
      <Anchor
        component="button"
        key={key}
        onClick={() => {
          onNavigate(crumb.path);
        }}
        size="sm"
        type="button"
      >
        {crumb.title}
      </Anchor>
    );
  });

  return (
    <MantineBreadcrumbs mt="xs" separator="→">
      {items}
    </MantineBreadcrumbs>
  );
}
