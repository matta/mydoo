/**
 * Type helper to strip index signatures (unknown keys) from `z.looseObject` types,
 * allowing enforcement of exhaustive destructuring on known keys.
 *
 * HOW IT WORKS:
 * It utilizes Key Remapping in Mapped Types.
 * `[K in keyof T as ...]` iterates over all keys.
 * `string extends K` checks if `K` is the generic string index signature (because specific string literal keys
 * like "id" are subtypes of string, but `string` itself is not a subtype of "id").
 * If it is the index signature, we map it to `never` (removing it).
 * Otherwise, we keep `K`.
 */
export type KnownKeysOnly<T> = {
  [K in keyof T as string extends K
    ? never
    : number extends K
      ? never
      : K]: T[K];
};
