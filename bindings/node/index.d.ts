/** Experimental P2.1 JSON transport. Full typed adapters belong to P2.2. */
export function analyze_json(request: string): string;
export class State {
  constructor(request: string);
  result_json(): string;
  identity_json(): string;
  update_json(update: string): string;
  reset_json(request: string): string;
  close(): void;
}
export class WycStateError extends Error { code: string; path: string; }
export class WycEngineError extends Error { code: string; path: string; }
