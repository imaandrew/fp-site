export async function getS3File(path: string): Promise<Uint8Array> {
  const url = `https://fp-web-patcher-files.s3.ca-central-1.amazonaws.com/${path}?x-id=GetObject`;
  const response = await fetch(url);
  const arrayBuffer = await response.arrayBuffer();
  const uint8Array = new Uint8Array(arrayBuffer);
  return uint8Array;
}
