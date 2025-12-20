import { RepoContext, useRepo } from "@automerge/automerge-repo-react-hooks";
import { repo } from "../lib/db";
import { ReactNode } from "react";

export function RepoProvider({ children }: { children: ReactNode }) {
  return <RepoContext.Provider value={repo}>{children}</RepoContext.Provider>;
}

export { useRepo };
