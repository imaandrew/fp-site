export class PatcherState {
  inputFile: File | null = $state(null);
  ver: string = $state("");
  tag: string = $state("");
  channelId: string = $state("");
  title: string = $state("");
  romHashMessage = $state("Choose base file");
  platform: string = $state("n64");
  returnZip = $state(false);
  enableDarkFilter = $state(false);
  enableWidescreen = $state(false);
  clickOutsideModal = $state(false);
  showLoading = $state(false);
  isVisible = $state(false);
  buttonText = $state("Build");
  alertText = $state("");
  reloadAlert = $state(0);

  outFileName: string = $derived.by(() => {
    if (this.tag === "" || this.ver === "" || this.ver === "unk") {
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

    return "";
  });

  disableButton: boolean = $derived(
    this.ver === "unk" ||
      this.ver === "" ||
      this.inputFile == null ||
      this.tag === "" ||
      this.outFileName === "" ||
      (this.platform === "wii" && (!this.title || !this.channelId)),
  );

  reset() {
    this.buttonText = "Build";
    this.showLoading = false;
  }
}
