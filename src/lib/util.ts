import { buf } from "crc-32";

export async function getS3File(path: string): Promise<Uint8Array> {
  const url = `https://fp-web-patcher-files.s3.ca-central-1.amazonaws.com/${path}?x-id=GetObject`;
  const response = await fetch(url);
  const arrayBuffer = await response.arrayBuffer();
  const uint8Array = new Uint8Array(arrayBuffer);
  return uint8Array;
}

export async function getTags(): Promise<string[]> {
  console.log("making request");
  const response = await fetch(
    "https://api.github.com/repos/jcog/fp/git/refs/tags"
  );
  const data = await response.json();
  return data.map((item: { ref: string }) => item.ref.split("/").pop());
}

export async function getCrc(file: File): Promise<number> {
  return readFileAsUint8Array(file).then((data) => {
    return buf(data) >>> 0;
  });
}

export function readFileAsUint8Array(file: File): Promise<Uint8Array> {
  return new Promise<Uint8Array>((resolve, reject) => {
    const reader = new FileReader();

    reader.onload = (event) => {
      const arrayBuffer = event.target?.result as ArrayBuffer;
      const uint8Array = new Uint8Array(arrayBuffer);
      resolve(uint8Array);
    };

    reader.onerror = (event) => {
      reject(event.target?.error);
    };

    reader.readAsArrayBuffer(file);
  });
}
