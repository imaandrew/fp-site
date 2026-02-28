export type Platform = "n64" | "wii" | "wiiu";

export class PatcherState {
  // Latest FP release
  ver: string = $state("");
  tag: string = $state("");

  // ROM file vars
  inputFile: File | null = $state(null);
  romHashMessage = $state("Choose base file");
  platform: Platform = $state("n64");

  // Wii options
  channelId: string = $state("");
  title: string = $state("");

  // WiiU options
  returnZip = $state(false);
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

  outFileName: string = $derived.by(() => {
    if (this.tag === "" || this.ver === "") {
      return "";
    }

    switch (this.platform) {
      case "n64":
        return `${this.tag}-${this.ver}.z64`;
      case "wii":
        return `${this.tag}-${this.ver}.wad`;
      case "wiiu":
        if (this.returnZip) {
          return `${this.tag}-${this.ver}.zip`;
        } else {
          return `${this.tag}-${this.ver}.tar`;
        }
    }
  });

  disableButton: boolean = $derived(
    this.ver === "" ||
      this.inputFile === null ||
      this.tag === "" ||
      this.outFileName === "" ||
      (this.platform === "wii" && (!this.title || !this.channelId)),
  );

  reset() {
    this.buttonText = "Build";
    this.showLoading = false;
  }

  showError(message: string) {
    this.alertText = message;
    this.isAlertVisible = true;
    this.reset();
  }
}
