export default {
  rules: {
    // 1. Check Subject Line (First line) <= 50 chars
    "header-max-length": [2, "always", 50],

    // 2. Check for long lines in the body <= 72 chars
    "body-max-line-length": [2, "always", 72],
  },
};
