import path from "node:path";

export const snapshotDir = path.join(
  import.meta.dirname,
  "../../playwright/.snapshots",
);
