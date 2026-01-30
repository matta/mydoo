import type { UserConfig } from "@commitlint/types";

const Configuration: UserConfig = {
  rules: {
    // Limits
    "header-max-length": [2, "always", 72],
    "header-min-length": [2, "always", 10], // Ensure meaningful messages
    "body-max-line-length": [2, "always", 72],

    // Formatting
    "header-full-stop": [2, "never", "."], // No period at end
    "body-leading-blank": [2, "always"], // Blank line before body
  },
};

export default Configuration;
