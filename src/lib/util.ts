import { buf } from "crc-32";

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
  node_id: string;
  object: {
    sha: string;
    type: string;
    url: string;
  };
  ref: string;
  url: string;
}

export async function getLatestTag(): Promise<string> {
  const response = await fetch(
    "https://api.github.com/repos/jcog/fp/git/refs/tags",
  );
  if (!response.ok) {
    throw new Error(`Error retrieving tag: ${response.statusText}`);
  }

  const data = (await response.json()) as Array<JSONResponse>;
  const tag = data[data.length - 1].ref.split("/").pop();
  if (tag == null) {
    throw new Error("Could not parse response");
  }
  return tag;
}

export async function getCrc(file: File): Promise<number> {
  const data = await readFileAsUint8Array(file);
  return buf(data) >>> 0;
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
      reject(new Error(event.target?.error?.message));
    };

    reader.readAsArrayBuffer(file);
  });
}
