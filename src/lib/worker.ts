export function setupWorkerHandler(
  worker: Worker,
  onSuccess: (data: Uint8Array) => void,
  onError: (message: string) => void,
) {
  worker.onmessage = (event: MessageEvent<Uint8Array | string>) => {
    if (typeof event.data === "string") {
      onError(event.data);
    } else {
      onSuccess(event.data);
    }
    worker.terminate();
  };
}
