const resultWindowPrefix = "result-";

export function isResultWindowLabel(label: string): boolean {
  return (
    label.startsWith(resultWindowPrefix) &&
    label.length > resultWindowPrefix.length
  );
}
