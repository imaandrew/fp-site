import type { PlatformOptions } from "./platform";

export class PatcherState {
  // Latest FP release
  tag: string = $state("");

  inputData: Uint8Array = $state(new Uint8Array());
  options: PlatformOptions | null = $state(null);

  // Alert options
  isAlertVisible = $state(false);
  alertText = $state("");

  building = $state(false);

  // Visual options
  infoModalOpen = $state(false);
  buttonText: string = $derived.by(() => {
    return this.building ? "Building..." : "Build";
  });

  outFileName: string = $derived.by(() => {
    if (this.tag === "" || this.options?.region == null) {
      return "";
    }

    switch (this.options.platform) {
      case "n64":
        return `${this.tag}-${this.options.region}.z64`;
      case "wii":
        return `${this.tag}-${this.options.region}.wad`;
      case "wiiu":
        return `${this.tag}-${this.options.region}.${this.options.resultType}`;
    }
  });

  disableButton: boolean = $derived(
    this.building ||
      this.options?.region == null ||
      this.inputData.length === 0 ||
      this.tag === "" ||
      this.outFileName === "",
  );

  reset() {
    this.building = false;
  }

  showError(message: string) {
    this.alertText = message;
    this.isAlertVisible = true;
    this.inputData = new Uint8Array();
    this.options = null;
    this.reset();
  }
}
