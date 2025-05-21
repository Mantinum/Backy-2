declare module '@tauri-apps/api/tauri' {
  /**
   * Invoke a Tauri command.
   * @param cmd The command to invoke.
   * @param args Optional arguments object.
   * @returns A promise resolving to the command's return value.
   */
  export function invoke<T = unknown>(
    cmd: string,
    args?: Record<string, unknown>
  ): Promise<T>;
}
