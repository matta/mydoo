import { configureProject } from "../../eslint.config.mjs";

export default [...configureProject(import.meta.dirname)];
