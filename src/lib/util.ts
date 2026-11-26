import { buf } from "crc-32";

import {
  ROM_HEADER_BYTESWAPPED,
  ROM_HEADER_LITTLEENDIAN,
} from "$lib/constants";

export async function getS3File(path: string): Promise<Uint8Array> {
  const response = await fetch(
    `https://fp-web-patcher-files.s3.ca-central-1.amazonaws.com/${path}?x-id=GetObject`,
  );
  if (!response.ok) {
    throw new Error(`Error retrieving s3 file: ${response.statusText}`);
  }

  return new Uint8Array(await response.arrayBuffer());
}

interface JSONResponse {
  tag_name: string;
}

export async function getLatestTag(): Promise<string> {
  const response = await fetch(
    "https://api.github.com/repos/jcog/fp/releases/latest",
  );
  if (!response.ok) {
    throw new Error(
      `Error retrieving tag from GitHub API: ${response.statusText}`,
    );
  }

  const data: JSONResponse = (await response.json()) as JSONResponse;
  const tag = data.tag_name;
  if (!tag) throw new Error("Invalid response from GitHub API");
  return tag;
}

export function getCrc(data: Uint8Array): number {
  return buf(data) >>> 0;
}

export async function readFileAsUint8Array(file: File): Promise<Uint8Array> {
  return new Uint8Array(await file.arrayBuffer());
}

export function swapBytes(input: Uint8Array) {
  const dataView = new DataView(input.buffer);
  const head = dataView.getUint32(0);
  if (head === ROM_HEADER_BYTESWAPPED) {
    for (let i = 0; i < dataView.byteLength; i += 2) {
      dataView.setUint16(i, dataView.getUint16(i), true);
    }
  } else if (head === ROM_HEADER_LITTLEENDIAN) {
    for (let i = 0; i < dataView.byteLength; i += 4) {
      dataView.setUint32(i, dataView.getUint32(i), true);
    }
  }
}

// https://stackoverflow.com/a/62176999
export function saveUint8ArrayToFile(uint8Array: Uint8Array, fileName: string) {
  const blob = new Blob([uint8Array as BlobPart], {
    type: "application/octet-stream",
  });
  const url = URL.createObjectURL(blob);

  const a = document.createElement("a");
  a.href = url;
  a.download = fileName;

  document.body.appendChild(a);
  a.click();

  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}
