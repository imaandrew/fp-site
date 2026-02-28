export interface PlatformPatcher {
  patch(input: Uint8Array, patchFile: Uint8Array): Promise<Worker>;
}
