import { createContext, useContext } from "react";
import { repo } from "../lib/db";

const RepoContext = createContext(repo);

export function RepoProvider({ children }: { children: React.ReactNode }) {
  return <RepoContext.Provider value={repo}>{children}</RepoContext.Provider>;
}

export function useRepo() {
  return useContext(RepoContext);
}
