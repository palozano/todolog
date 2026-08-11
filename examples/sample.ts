export function normalizeTitle(title: string): string {
  // TODO: preserve known acronyms when title-casing.
  return title.trim().toLowerCase();
}

export function parseDate(input: string): Date {
  // FIXME: reject invalid dates instead of relying on Date coercion.
  return new Date(input);
}

export const lorem = [
  "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
  "Praesent dapibus, neque id cursus faucibus, tortor neque egestas augue.",
].join(" ");
