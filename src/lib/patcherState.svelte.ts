import type { Region, ResultType } from "$wasm/fp_web_patcher";

export type Platform = "n64" | "wii" | "wiiu";

export class PatcherState {
  // Latest FP release
  region: Region | null = $state(null);
  tag: string = $state("");

  // ROM file vars
  inputFile: File | null = $state(null);
  romHashMessage = $state("Choose base file");
  platform: Platform | null = $state(null);

  // Wii options
  channelId: string = $state("");
  title: string = $state("");

  // WiiU options
  resultType: ResultType = $state("zip");
  enableDarkFilter = $state(false);
  enableWidescreen = $state(false);

  // Visual options
  clickOutsideModal = $state(false);
  showLoading = $state(false);
  buttonText = $state("Build");

  // Alert options
  isAlertVisible = $state(false);
  alertText = $state("");
  reloadAlert = $state(0);

  building = $state(false);

  outFileName: string = $derived.by(() => {
    if (this.tag === "" || this.region === null || this.platform === null) {
      return "";
    }

    switch (this.platform) {
      case "n64":
        return `${this.tag}-${this.region}.z64`;
      case "wii":
        return `${this.tag}-${this.region}.wad`;
      case "wiiu":
        return `${this.tag}-${this.region}.${this.resultType}`;
    }
  });

  disableButton: boolean = $derived(
    this.building ||
      this.region === null ||
      this.inputFile === null ||
      this.tag === "" ||
      this.outFileName === "" ||
      (this.platform === "wii" && (!this.title || !this.channelId)),
  );

  reset() {
    this.buttonText = "Build";
    this.showLoading = false;
    this.building = false;
  }

  showError(message: string) {
    this.alertText = message;
    this.isAlertVisible = true;
    this.reset();
  }
}
