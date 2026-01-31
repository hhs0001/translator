declare module 'bun:test' {
  export const beforeEach: (fn: () => void | Promise<void>) => void;
  export const test: (name: string, fn: () => void | Promise<void>) => void;
  export const expect: <T = unknown>(actual: T) => any;
  export const mock: {
    module: (path: string, factory: () => Record<string, unknown>) => void;
  };
}
