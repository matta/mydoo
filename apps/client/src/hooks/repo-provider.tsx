import { RepoContext } from '@automerge/automerge-repo-react-hooks';
import type { ReactNode } from 'react';

import { repo } from '../lib/db';

export function RepoProvider({ children }: { children: ReactNode }) {
  return <RepoContext.Provider value={repo}>{children}</RepoContext.Provider>;
}
