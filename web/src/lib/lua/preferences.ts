import { readStored, writeStored } from "@/lib/storage";

const DRY_RUN = "home-gateway.lua.dry-run";
const SIDEBAR = "home-gateway.lua.sidebar";

export function readDryRun(): boolean {
  return readStored(DRY_RUN, true);
}

export function writeDryRun(value: boolean): void {
  writeStored(DRY_RUN, value);
}

export function readSidebar(): boolean {
  return readStored(SIDEBAR, true);
}

export function writeSidebar(value: boolean): void {
  writeStored(SIDEBAR, value);
}
