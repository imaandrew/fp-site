import { ParallelHasher } from "ts-md5";

export function getMd5(file: File): Promise<string> {
  const hasher = new ParallelHasher(
    "../../node_modules/ts-md5/dist/md5_worker.js"
  );
  const x = hasher.hash(file).then(function (result: string) {
    return result;
  });

  return x;
}
