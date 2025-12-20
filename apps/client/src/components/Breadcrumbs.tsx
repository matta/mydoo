interface BreadcrumbItem {
  id: string;
  title: string;
  path: string[];
}

interface BreadcrumbsProps {
  crumbs: BreadcrumbItem[];
  onNavigate: (path: string[]) => void;
}

import { Breadcrumbs as MantineBreadcrumbs, Anchor, Text } from "@mantine/core";

export function Breadcrumbs({ crumbs, onNavigate }: BreadcrumbsProps) {
  const items = crumbs.map((crumb, index) => {
    const isLast = index === crumbs.length - 1;

    if (isLast) {
      return (
        <Text key={crumb.id} c="dimmed">
          {crumb.title}
        </Text>
      );
    }

    return (
      <Anchor
        key={crumb.id}
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
