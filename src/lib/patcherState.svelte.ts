export class PatcherState {
  inputFile: File | null = $state(null);
  ver: string = $state("");
  tag: string = $state("");
  channelId: string = $state("");
  title: string = $state("");
  romHashMessage = $state("Choose base file");
  outFileName: string = $state("");
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

  disableButton: boolean = $derived(
    this.ver === "unk" ||
      this.ver === "" ||
      this.inputFile == null ||
      this.tag == null ||
      this.tag === "" ||
      this.outFileName == null ||
      this.outFileName === "" ||
      (this.platform === "wii" && (!this.title || !this.channelId)),
  );

  reset() {
    this.buttonText = "Build";
    this.showLoading = false;
  }
}
