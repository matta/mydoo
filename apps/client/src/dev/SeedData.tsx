import type {DocumentHandle} from '@mydoo/tasklens';
import {useTunnel} from '@mydoo/tasklens';
import {useEffect, useRef} from 'react';

/**
 * Helper component that seeds data when the `?seed=true` query param is present.
 *
 * Usage: <SeedData docUrl={docUrl} />
 */
export function SeedData({docUrl}: {docUrl: DocumentHandle}) {
  const {doc, ops} = useTunnel(docUrl);
  const seeded = useRef(false);

  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    if (params.get('seed') === 'true' && doc && !seeded.current) {
      const taskCount = Object.keys(doc.tasks).length;
      if (taskCount === 0) {
        seeded.current = true;
        ops.add({title: 'Buy Milk', priority: 1, importance: 1});
        ops.add({title: 'Walk Dog', priority: 0.5, importance: 0.5});
        ops.add({title: 'Read Book', priority: 0.1, importance: 0.1});
      }
    }
  }, [doc, ops]);

  // biome-ignore lint/complexity/noUselessFragments: unblocks TS build
  return <></>;
}
