import type { Region, ResultType } from "$wasm/fp_web_patcher";

import { ROM_CRC } from "./constants";

export type Platform = "n64" | "wii" | "wiiu";

export interface N64Options {
  platform: "n64";
  region: Region;
  romHashMessage: string;
}

export interface WiiOptions {
  platform: "wii";
  region: Region;
  romHashMessage: string;
  channelId: string;
  title: string;
}

export interface WiiUOptions {
  platform: "wiiu";
  region: Region | null;
  romHashMessage: string;
  resultType: ResultType;
  enableDarkFilter: boolean;
  enableWidescreen: boolean;
}

export type PlatformOptions = N64Options | WiiOptions | WiiUOptions;

export function detectOptions(
  crc: number,
  fileName: string,
): PlatformOptions | null {
  switch (crc) {
    case ROM_CRC.US_Z64:
    case ROM_CRC.US_V64:
    case ROM_CRC.US_N64:
      return {
        platform: "n64",
        region: "us",
        romHashMessage: "Valid US ROM",
      };
    case ROM_CRC.JP_Z64:
    case ROM_CRC.JP_V64:
    case ROM_CRC.JP_N64:
      return {
        platform: "n64",
        region: "jp",
        romHashMessage: "Valid JP ROM",
      };
    case ROM_CRC.US_WAD:
      return {
        platform: "wii",
        region: "us",
        romHashMessage: "Valid US WAD",
        channelId: "FPUS",
        title: "fp-us",
      };
    case ROM_CRC.JP_WAD:
      return {
        platform: "wii",
        region: "jp",
        romHashMessage: "Valid JP WAD",
        channelId: "FPJP",
        title: "fp-jp",
      };
    default: {
      const ext = fileName.toLowerCase().split(".").pop();
      if (ext !== "zip" && ext !== "tar") return null;
      return {
        platform: "wiiu",
        region: null,
        romHashMessage: "Wii U Archive",
        resultType: ext,
        enableDarkFilter: false,
        enableWidescreen: false,
      };
    }
  }
}
